#[test]
fn test_parse_response() {
    let mut input: Vec<(&str, &str)> = vec![
        (" {{Hero icon|Abaddon|16px}} [[File:Mist Coil icon.png|16px|link=Mist Coil]] Reclaimed for Avernus.", "Reclaimed for Avernus."),
        ("{{resp|u}} [[Shitty Wizard]]!", "Shitty Wizard!"),
        (" From House Avernus, I set forth.", "From House Avernus, I set forth."),
        (" I have known from an early age, since the Avernal Font's first and greatest revelation, that this one would be in the bag.", "I have known from an early age, since the Avernal Font's first and greatest revelation, that this one would be in the bag."),
        ("Who goes there?", "Who goes there?"),
        (" The [[fog of war]] is no match for the mist of fate.", "The fog of war is no match for the mist of fate."),
        ("<small>remove this text</small> response here.", "response here."),
        (" {{resp|60}} This magic…disappoints.", "This magic…disappoints."),
        ("{{resp|60}} My concentration—shattered!", "My concentration—shattered!"),
        ( " [nonverbal]", "nonverbal"),
    ];

    for (mut input, expected) in input.iter_mut() {
        match crate::dota::parsing::parse_response(&mut input) {
            Ok(response) => {
                assert_eq!(&response, expected);
            }
            Err(e) => {
                panic!("{:?}", e);
            }
        }
    }
}

#[test]
fn test_parse_response_line() {
    let mut input: Vec<(&str, (&str, &str))> = vec![(
        "* <sm2>vo_abaddon_abad_spawn_01.mp3</sm2> Abaddon.",
        ("Vo abaddon abad spawn 01.mp3", "Abaddon."),
    )];

    for (mut input, (expected_file, expected_response)) in input.iter_mut() {
        match crate::dota::parsing::parse_response_line(&mut input) {
            Ok((file, response)) => {
                assert_eq!(file, expected_file.to_string());
                assert_eq!(response, expected_response.to_string());
            }
            Err(e) => {
                panic!("{:?}", e);
            }
        }
    }
}

#[test]
/// Input should appear with literal \n rather than newline character
fn test_all_response_lines() {
    let mut input = r#"\n{{Tabs Hero}}\n\n{{VoiceNavSidebar}}{{TOC float|right}}\n\n== Loadout ==\n* <sm2>vo_abaddon_abad_spawn_01.mp3</sm2> Abaddon.\n\n== Drafting ==\n''Picked''\n* <sm2>vo_abaddon_abad_spawn_01.mp3</sm2> Abaddon.\n"#;

    let expected = [
        ("Vo abaddon abad spawn 01.mp3", "Abaddon."),
        ("Vo abaddon abad spawn 01.mp3", "Abaddon."),
    ];

    match crate::dota::parsing::parse_all_response_lines(&mut input) {
        Ok(response_lines) => {
            assert_eq!(response_lines.len(), expected.len());
            for (actual, expected) in response_lines.iter().zip(expected.iter()) {
                assert_eq!(actual.file, expected.0);
                assert_eq!(actual.response, expected.1);
            }
        }
        Err(e) => {
            panic!("{:?}", e);
        }
    }
}
