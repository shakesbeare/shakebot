use winnow::{
    combinator::{alt, delimited, seq},
    error::{AddContext as _, ContextError, ErrMode, StrContext},
    prelude::*,
    token::{literal, take, take_until, take_while},
};

use regex::Regex;

#[derive(Debug, PartialEq, Clone)]
pub struct Response {
    pub file: String,
    pub response: String,
}

#[derive(Debug, PartialEq, Clone)]
enum SquareTag<'a> {
    Ignorable,
    ContainsResponse(&'a str),
}

#[derive(Debug, PartialEq, Clone)]
enum ResponseKind {
    Standard,
    Vgs,
}

fn parse_vertical_bar<'a>(input: &mut &'a str) -> ModalResult<&'a str> {
    literal("|")
        .context(StrContext::Label("vertical bar"))
        .parse_next(input)
}

fn parse_new_line<'a>(input: &mut &'a str) -> ModalResult<&'a str> {
    literal("\n")
        .context(StrContext::Label("newline"))
        .parse_next(input)
}

fn parse_squirly_tag<'a>(input: &mut &'a str) -> ModalResult<&'a str> {
    // always ignored
    let capture = delimited(r"{{", take_until(0.., r"}}"), r"}}")
        .context(StrContext::Label("squirly delimiter"))
        .parse_next(input)?;
    ignore_space.parse_next(input)?;
    Ok(capture)
}

fn parse_square_tag<'a>(input: &mut &'a str) -> ModalResult<SquareTag<'a>> {
    // link style tags (ie [[ text ]] ) should be used as part of the response
    // file style tags (ie [[ text | something ]]) should be ignored
    let capture = delimited(r"[[", take_until(0.., r"]]"), r"]]")
        .context(StrContext::Label("square tag delimiters"))
        .parse_next(input)?;
    if capture.contains('|') {
        ignore_space.parse_next(input)?;
        Ok(SquareTag::Ignorable)
    } else {
        Ok(SquareTag::ContainsResponse(capture))
    }
}

fn parse_single_square_tag<'a>(input: &mut &'a str) -> ModalResult<&'a str> {
    delimited(r"[", take_until(0.., r"]"), r"]")
        .context(StrContext::Label("single square delimiter"))
        .parse_next(input)
}

fn parse_angle_tag_open<'a>(input: &mut &'a str) -> ModalResult<&'a str> {
    let capture = delimited(r"<", take_until(0.., r">"), r">")
        .context(StrContext::Label("angle open delimiter"))
        .parse_next(input)?;
    ignore_space.parse_next(input)?;
    Ok(capture)
}

fn parse_angle_tag_close<'a>(input: &mut &'a str) -> ModalResult<&'a str> {
    let capture = delimited(r"</", take_until(0.., r">"), r">")
        .context(StrContext::Label("angle close delimiter"))
        .parse_next(input)?;
    ignore_space.parse_next(input)?;
    Ok(capture)
}

fn parse_complete_angle_tag<'a>(input: &mut &'a str) -> ModalResult<&'a str> {
    delimited(
        parse_angle_tag_open.context(StrContext::Label("open tag")),
        take_until(0.., r"<").context(StrContext::Label("tag content")),
        parse_angle_tag_close.context(StrContext::Label("close tag")),
    )
    .parse_next(input)
}

fn parse_angle_tag<'a>(input: &mut &'a str) -> ModalResult<&'a str> {
    // could be of the form <tag> or <tag>ignored text</tag>
    alt((parse_complete_angle_tag, parse_angle_tag_open))
        .context(StrContext::Label("angle tag"))
        .parse_next(input)
}

fn parse_quote<'a>(input: &mut &'a str) -> ModalResult<&'a str> {
    take_while(0.., |c: char| !['{', '[', '<', '\n', '\r'].contains(&c))
        .context(StrContext::Label("quote"))
        .parse_next(input)
}

fn ignore_space<'a>(input: &mut &'a str) -> ModalResult<&'a str> {
    take_while(0.., ' ')
        .context(StrContext::Label("ignore space"))
        .parse_next(input)
}

pub fn parse_response(input: &mut &str) -> ModalResult<String> {
    ignore_space.parse_next(input)?;
    let mut response = String::new();
    while !input.is_empty() {
        if input.starts_with("{{") {
            parse_squirly_tag.parse_next(input)?;
        } else if input.starts_with("[[") {
            let tag = parse_square_tag.parse_next(input)?;
            match tag {
                SquareTag::Ignorable => (),
                SquareTag::ContainsResponse(quote_fragment) => response.push_str(quote_fragment),
            }
        } else if input.starts_with('[') {
            let tag = parse_single_square_tag.parse_next(input)?;
            response.push_str(tag);
        } else if input.starts_with('<') {
            parse_angle_tag.parse_next(input)?;
        } else {
            let quote_fragment = parse_quote.parse_next(input)?;
            response.push_str(quote_fragment);
        }
    }

    Ok(response)
}

fn parse_begin_line<'a>(input: &mut &'a str) -> ModalResult<ResponseKind> {
    if input.starts_with('*') {
        take_until(0.., r"<")
            .context(StrContext::Label("begin line"))
            .parse_next(input)?;
        Ok(ResponseKind::Standard)
    } else {
        seq!(
            parse_vertical_bar,
            take_until(0.., '|'),
            parse_vertical_bar,
            ignore_space
        )
        .context(StrContext::Label("begin line"))
        .parse_next(input)?;
        Ok(ResponseKind::Vgs)
    }
}

pub fn parse_response_line(input: &mut &str) -> ModalResult<Vec<(String, String)>> {
    match parse_begin_line.parse_next(input)? {
        ResponseKind::Standard => {
            let file = parse_complete_angle_tag
                .context(StrContext::Label("parse sm2 tag"))
                .parse_next(input)?;
            ignore_space.parse_next(input)?;
            let response = parse_response(input)?;

            // process filename
            let mut file = file.replace('_', " ").chars().collect::<Vec<char>>();
            file[0] = file[0].to_uppercase().next().unwrap();
            let file = file.into_iter().collect::<String>();
            Ok(vec![(file, response)])
        }
        ResponseKind::Vgs => {
            let mut has_alternate = false;
            if input.contains("<br>") {
                has_alternate = true;
            }

            if has_alternate {
                let first_response = take_until(0.., '<')
                    .context(StrContext::Label("response"))
                    .context(StrContext::Label("Parse VGS response text"))
                    .parse_next(input)?
                    .to_string();

                parse_angle_tag
                    .context(StrContext::Label("expected <br>"))
                    .parse_next(input)?;
                ignore_space(input)?;

                let second_response = take_until(0.., '|')
                    .context(StrContext::Label("response"))
                    .context(StrContext::Label("Parse VGS response text"))
                    .parse_next(input)?
                    .to_string();

                parse_vertical_bar(input)?;
                ignore_space(input)?;

                let first_file = parse_complete_angle_tag
                    .context(StrContext::Label("parse sm2 tag"))
                    .parse_next(input)?;

                parse_angle_tag
                    .context(StrContext::Label("expected <br>"))
                    .parse_next(input)?;

                let second_file = parse_complete_angle_tag
                    .context(StrContext::Label("parse sm2 tag"))
                    .parse_next(input)?;

                Ok(vec![
                    (first_file.to_string(), first_response),
                    (second_file.to_string(), second_response),
                ])
            } else {
                let response = take_until(0.., '|')
                    .context(StrContext::Label("response"))
                    .context(StrContext::Label("Parse VGS response text"))
                    .parse_next(input)?
                    .to_string();
                parse_vertical_bar(input)?;
                ignore_space(input)?;

                let file = parse_complete_angle_tag
                    .context(StrContext::Label("parse sm2 tag"))
                    .parse_next(input)?;

                Ok(vec![(file.to_string(), response)])
            }
        }
    }
}

pub fn parse_all_response_lines(input: &mut &str) -> ModalResult<Vec<Response>> {
    // turn input into a vec of lines
    // let input = input.lines().collect::<Vec<&str>>();
    let re = Regex::new(r"\n|\\n|\r|\\r").unwrap();
    let mut input = re.split(input);
    let mut responses = Vec::new();
    while let Some(mut line) = input.next() {
        // check if line starts with <sm2>
        if line.starts_with("* <sm2>") {
            // if it does, parse the line
            let parsed = parse_response_line.parse_next(&mut line)?;
            for (file, response) in parsed {
                responses.push(Response { file, response });
            }
        } else if line.starts_with("| align=\"left\" | \"") {
            // these lines contain information over two lines
            let cur_line = line;
            let next_line = input.next().unwrap();
            let mut merged = cur_line.to_string();
            merged.push_str(next_line);
            let parsed = parse_response_line.parse_next(&mut merged.as_str())?;
            for (file, response) in parsed {
                responses.push(Response { file, response });
            }
        }
    }

    Ok(responses)
}
