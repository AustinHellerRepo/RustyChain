use std::collections::HashMap;

use madlib::{SpeechPart, MadlibConstructor, MadlibConstructionInitializer};


mod madlib {
    use std::collections::HashMap;

    use rand::seq::SliceRandom;
    use rusty_chain::{chain_link, chain};

    pub enum MadlibPart {
        Static(String),
        Dynamic(SpeechPart),
        End
    }

    #[derive(PartialEq, Hash, Eq, Clone)]
    pub enum SpeechPart {
        Noun,
        Verb,
        Adjective
    }

    pub enum ConstructedMadlibPart {
        Word(String),
        End
    }

    // TODO make the possible_words_per_speech_part purely part of the construction and pass in a vector of madlib parts

    chain_link!(MadlibConstruction => (
            madlib_parts: Vec<MadlibPart>,
            index: usize,
            possible_words_per_speech_part: Option<HashMap<SpeechPart, Vec<String>>>
        ),
        input: HashMap<SpeechPart, Vec<String>> => ConstructedMadlibPart, {

        if let Some(possible_words_per_speech_part) = input.received {
            input.initializer.lock().await.possible_words_per_speech_part.replace(possible_words_per_speech_part.clone());
        }

        let mut locked_initializer = input.initializer.lock().await;
        if let Some(possible_words_per_speech_part) = locked_initializer.possible_words_per_speech_part.as_ref() {
            if locked_initializer.index == locked_initializer.madlib_parts.len() {
                None
            }
            else
            {
                let index = locked_initializer.index;
                let word = match &locked_initializer.madlib_parts[index] {
                    MadlibPart::Static(text) => {
                        ConstructedMadlibPart::Word(text.clone())
                    },
                    MadlibPart::Dynamic(part) => {
                        let possible_words = possible_words_per_speech_part.get(&part).unwrap();
                        ConstructedMadlibPart::Word(possible_words.choose(&mut rand::thread_rng()).unwrap().clone())
                    },
                    MadlibPart::End => {
                        ConstructedMadlibPart::End
                    }
                };
                locked_initializer.index += 1;
                Some(word)
            }
        }
        else {
            None
        }
    });

    chain_link!(CollectConstructedMadlibParts => ( buffer: Vec<String> ), input: ConstructedMadlibPart => String, {
        match input.received {
            Some(constructed_madlib_part) => {
                match constructed_madlib_part {
                    ConstructedMadlibPart::End => {
                        let output = Some(input.initializer.lock().await.buffer.join(" "));
                        input.initializer.lock().await.buffer.clear();
                        output
                    },
                    ConstructedMadlibPart::Word(word) => {
                        input.initializer.lock().await.buffer.push(word.clone());
                        None
                    }
                }
            },
            None => None
        }
    });

    chain!(MadlibConstructor, HashMap<SpeechPart, Vec<String>> => String, MadlibConstruction => CollectConstructedMadlibParts);
}

#[tokio::main]
async fn main() {
    let mut possible_words_per_speech_part: HashMap<SpeechPart, Vec<String>> = HashMap::new();
    possible_words_per_speech_part.insert(SpeechPart::Noun, vec![
        String::from("door"),
        String::from("cat"),
        String::from("dog")
    ]);
    possible_words_per_speech_part.insert(SpeechPart::Verb, vec![
        String::from("run"),
        String::from("walk"),
        String::from("eat")
    ]);
    possible_words_per_speech_part.insert(SpeechPart::Adjective, vec![
        String::from("green"),
        String::from("hard"),
        String::from("sour")
    ]);
    
    let madlib_constructor = MadlibConstructor::new(madlib::MadlibConstructorInitializer { x_madlib_construction: MadlibConstructionInitializer { }, xx_collect_constructed_madlib_parts: () })
}