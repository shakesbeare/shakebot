use winnow::{
    combinator::{alt, delimited},
    error::StrContext,
    prelude::*,
    token::{take_until, take_while},
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

fn parse_squirly_tag<'a>(input: &mut &'a str) -> PResult<&'a str> {
    // always ignored
    let capture = delimited(r"{{", take_until(0.., r"}}"), r"}}")
        .context(StrContext::Label("squirly delimiter"))
        .parse_next(input)?;
    ignore_space.parse_next(input)?;
    Ok(capture)
}

fn parse_square_tag<'a>(input: &mut &'a str) -> PResult<SquareTag<'a>> {
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

fn parse_single_square_tag<'a>(input: &mut &'a str) -> PResult<&'a str> {
    delimited(r"[", take_until(0.., r"]"), r"]")
        .context(StrContext::Label("single square delimiter"))
        .parse_next(input)
}

fn parse_angle_tag_open<'a>(input: &mut &'a str) -> PResult<&'a str> {
    let capture = delimited(r"<", take_until(0.., r">"), r">")
        .context(StrContext::Label("angle open delimiter"))
        .parse_next(input)?;
    ignore_space.parse_next(input)?;
    Ok(capture)
}

fn parse_angle_tag_close<'a>(input: &mut &'a str) -> PResult<&'a str> {
    let capture = delimited(r"</", take_until(0.., r">"), r">")
        .context(StrContext::Label("angle close delimiter"))
        .parse_next(input)?;
    ignore_space.parse_next(input)?;
    Ok(capture)
}

fn parse_complete_angle_tag<'a>(input: &mut &'a str) -> PResult<&'a str> {
    delimited(
        parse_angle_tag_open.context(StrContext::Label("open tag")),
        take_until(0.., r"<").context(StrContext::Label("tag content")),
        parse_angle_tag_close.context(StrContext::Label("close tag")),
    )
    .parse_next(input)
}

fn parse_angle_tag<'a>(input: &mut &'a str) -> PResult<&'a str> {
    // could be of the form <tag> or <tag>ignored text</tag>
    alt((parse_complete_angle_tag, parse_angle_tag_open))
        .context(StrContext::Label("angle tag"))
        .parse_next(input)
}

fn parse_quote<'a>(input: &mut &'a str) -> PResult<&'a str> {
    take_while(0.., |c: char| !['{', '[', '<', '\n', '\r'].contains(&c))
        .context(StrContext::Label("quote"))
        .parse_next(input)
}

fn ignore_space<'a>(input: &mut &'a str) -> PResult<&'a str> {
    take_while(0.., ' ')
        .context(StrContext::Label("ignore space"))
        .parse_next(input)
}

pub fn parse_response(input: &mut &str) -> PResult<String> {
    ignore_space.parse_next(input)?;
    let mut response = String::new();
    while !input.is_empty() {
        if input.starts_with("{{") {
            parse_squirly_tag.parse_next(input)?;
        } else if input.starts_with("[[") {
            let tag = parse_square_tag.parse_next(input)?;
            match tag {
                SquareTag::Ignorable => (),
                SquareTag::ContainsResponse(quote_fragment) => {
                    response.push_str(quote_fragment)
                }
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

fn parse_begin_line<'a>(input: &mut &'a str) -> PResult<&'a str> {
    take_until(0.., r"<")
        .context(StrContext::Label("begin line"))
        .parse_next(input)
}

pub fn parse_response_line(input: &mut &str) -> PResult<(String, String)> {
    parse_begin_line.parse_next(input)?;
    let file = parse_complete_angle_tag
        .context(StrContext::Label("parse sm1 tag"))
        .parse_next(input)?;
    ignore_space.parse_next(input)?;
    let response = parse_response(input)?;

    // process filename
    let mut file = file.replace('_', " ").chars().collect::<Vec<char>>();
    file[0] = file[0].to_uppercase().next().unwrap();
    let file = file.into_iter().collect::<String>();
    Ok((file, response))
}

pub fn parse_all_response_lines(input: &mut &str) -> PResult<Vec<Response>> {
    // turn input into a vec of lines
    // let input = input.lines().collect::<Vec<&str>>();
    let re = Regex::new(r"\n|\\n").unwrap();
    let input = re.split(input);
    let mut responses = Vec::new();
    for mut line in input {
        // check if line starts with <sm2>
        if line.starts_with("* <sm2>") {
            // if it does, parse the line
            let (file, response) = parse_response_line.parse_next(&mut line)?;
            responses.push(Response { file, response });
        }
    }

    Ok(responses)
}
