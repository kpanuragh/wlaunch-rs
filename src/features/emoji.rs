use crate::core::{Item, ItemType};

pub struct EmojiManager {
    emojis: Vec<(&'static str, &'static str, Vec<&'static str>)>,
}

impl EmojiManager {
    pub fn new() -> Self {
        Self {
            emojis: vec![
                // Smileys
                ("😀", "grinning face", vec!["smile", "happy", "grin"]),
                ("😃", "grinning face with big eyes", vec!["smile", "happy"]),
                ("😄", "grinning face with smiling eyes", vec!["smile", "happy"]),
                ("😁", "beaming face with smiling eyes", vec!["smile", "happy", "grin"]),
                ("😆", "grinning squinting face", vec!["laugh", "happy"]),
                ("😅", "grinning face with sweat", vec!["sweat", "nervous"]),
                ("🤣", "rolling on the floor laughing", vec!["rofl", "laugh", "lol"]),
                ("😂", "face with tears of joy", vec!["laugh", "cry", "lol", "joy"]),
                ("🙂", "slightly smiling face", vec!["smile"]),
                ("🙃", "upside-down face", vec!["silly", "sarcasm"]),
                ("😉", "winking face", vec!["wink"]),
                ("😊", "smiling face with smiling eyes", vec!["blush", "smile"]),
                ("😇", "smiling face with halo", vec!["angel", "innocent"]),
                ("🥰", "smiling face with hearts", vec!["love", "hearts"]),
                ("😍", "smiling face with heart-eyes", vec!["love", "heart", "eyes"]),
                ("🤩", "star-struck", vec!["star", "eyes", "wow"]),
                ("😘", "face blowing a kiss", vec!["kiss", "love"]),
                ("😗", "kissing face", vec!["kiss"]),
                ("😚", "kissing face with closed eyes", vec!["kiss"]),
                ("😙", "kissing face with smiling eyes", vec!["kiss"]),
                ("🥲", "smiling face with tear", vec!["happy", "sad", "tear"]),
                ("😋", "face savoring food", vec!["yum", "delicious"]),
                ("😛", "face with tongue", vec!["tongue"]),
                ("😜", "winking face with tongue", vec!["tongue", "wink", "silly"]),
                ("🤪", "zany face", vec!["crazy", "silly", "wild"]),
                ("😝", "squinting face with tongue", vec!["tongue"]),
                ("🤑", "money-mouth face", vec!["money", "rich"]),
                ("🤗", "hugging face", vec!["hug"]),
                ("🤭", "face with hand over mouth", vec!["oops", "shy"]),
                ("🤫", "shushing face", vec!["quiet", "shh", "secret"]),
                ("🤔", "thinking face", vec!["think", "hmm"]),
                ("🤐", "zipper-mouth face", vec!["quiet", "zip"]),
                ("🤨", "face with raised eyebrow", vec!["skeptical", "doubt"]),
                ("😐", "neutral face", vec!["neutral", "meh"]),
                ("😑", "expressionless face", vec!["blank", "meh"]),
                ("😶", "face without mouth", vec!["silent", "speechless"]),
                ("😏", "smirking face", vec!["smirk"]),
                ("😒", "unamused face", vec!["unamused", "meh"]),
                ("🙄", "face with rolling eyes", vec!["eyeroll", "whatever"]),
                ("😬", "grimacing face", vec!["grimace", "awkward"]),
                ("🤥", "lying face", vec!["lie", "pinocchio"]),
                ("😌", "relieved face", vec!["relieved", "peaceful"]),
                ("😔", "pensive face", vec!["sad", "pensive"]),
                ("😪", "sleepy face", vec!["sleepy", "tired"]),
                ("🤤", "drooling face", vec!["drool", "yum"]),
                ("😴", "sleeping face", vec!["sleep", "zzz"]),
                ("😷", "face with medical mask", vec!["mask", "sick"]),
                ("🤒", "face with thermometer", vec!["sick", "fever"]),
                ("🤕", "face with head-bandage", vec!["hurt", "injured"]),
                ("🤢", "nauseated face", vec!["sick", "green"]),
                ("🤮", "face vomiting", vec!["sick", "vomit"]),
                ("🤧", "sneezing face", vec!["sneeze", "sick"]),
                ("🥵", "hot face", vec!["hot", "heat"]),
                ("🥶", "cold face", vec!["cold", "freeze"]),
                ("🥴", "woozy face", vec!["drunk", "dizzy"]),
                ("😵", "dizzy face", vec!["dizzy"]),
                ("🤯", "exploding head", vec!["mind blown", "shocked"]),
                ("🤠", "cowboy hat face", vec!["cowboy"]),
                ("🥳", "partying face", vec!["party", "celebrate"]),
                ("🥸", "disguised face", vec!["disguise", "incognito"]),
                ("😎", "smiling face with sunglasses", vec!["cool", "sunglasses"]),
                ("🤓", "nerd face", vec!["nerd", "geek"]),
                ("🧐", "face with monocle", vec!["monocle", "fancy"]),
                ("😕", "confused face", vec!["confused"]),
                ("😟", "worried face", vec!["worried"]),
                ("🙁", "slightly frowning face", vec!["frown", "sad"]),
                ("☹️", "frowning face", vec!["frown", "sad"]),
                ("😮", "face with open mouth", vec!["surprised", "wow"]),
                ("😯", "hushed face", vec!["surprised", "hushed"]),
                ("😲", "astonished face", vec!["astonished", "shocked"]),
                ("😳", "flushed face", vec!["blush", "embarrassed"]),
                ("🥺", "pleading face", vec!["puppy", "please", "beg"]),
                ("😦", "frowning face with open mouth", vec!["frown"]),
                ("😧", "anguished face", vec!["anguished"]),
                ("😨", "fearful face", vec!["fear", "scared"]),
                ("😰", "anxious face with sweat", vec!["anxious", "nervous"]),
                ("😥", "sad but relieved face", vec!["sad", "relieved"]),
                ("😢", "crying face", vec!["cry", "sad"]),
                ("😭", "loudly crying face", vec!["cry", "sob", "sad"]),
                ("😱", "face screaming in fear", vec!["scream", "fear", "omg"]),
                ("😖", "confounded face", vec!["confounded"]),
                ("😣", "persevering face", vec!["persevere"]),
                ("😞", "disappointed face", vec!["disappointed", "sad"]),
                ("😓", "downcast face with sweat", vec!["sweat"]),
                ("😩", "weary face", vec!["weary", "tired"]),
                ("😫", "tired face", vec!["tired"]),
                ("🥱", "yawning face", vec!["yawn", "tired", "bored"]),
                ("😤", "face with steam from nose", vec!["angry", "frustrated"]),
                ("😡", "pouting face", vec!["angry", "mad"]),
                ("😠", "angry face", vec!["angry", "mad"]),
                ("🤬", "face with symbols on mouth", vec!["swear", "angry", "curse"]),
                ("😈", "smiling face with horns", vec!["devil", "evil"]),
                ("👿", "angry face with horns", vec!["devil", "angry"]),
                ("💀", "skull", vec!["death", "dead"]),
                ("☠️", "skull and crossbones", vec!["death", "danger"]),
                ("💩", "pile of poo", vec!["poop", "crap"]),
                ("🤡", "clown face", vec!["clown"]),
                ("👹", "ogre", vec!["monster", "ogre"]),
                ("👺", "goblin", vec!["monster", "goblin"]),
                ("👻", "ghost", vec!["ghost", "boo"]),
                ("👽", "alien", vec!["alien", "ufo"]),
                ("👾", "alien monster", vec!["alien", "game"]),
                ("🤖", "robot", vec!["robot", "bot"]),

                // Gestures
                ("👋", "waving hand", vec!["wave", "hello", "bye"]),
                ("🤚", "raised back of hand", vec!["hand"]),
                ("🖐️", "hand with fingers splayed", vec!["hand", "five"]),
                ("✋", "raised hand", vec!["stop", "hand", "high five"]),
                ("🖖", "vulcan salute", vec!["spock", "vulcan"]),
                ("👌", "OK hand", vec!["ok", "perfect"]),
                ("🤌", "pinched fingers", vec!["italian", "chef"]),
                ("🤏", "pinching hand", vec!["small", "tiny"]),
                ("✌️", "victory hand", vec!["peace", "victory"]),
                ("🤞", "crossed fingers", vec!["luck", "hope"]),
                ("🤟", "love-you gesture", vec!["love", "rock"]),
                ("🤘", "sign of the horns", vec!["rock", "metal"]),
                ("🤙", "call me hand", vec!["call", "shaka"]),
                ("👈", "backhand index pointing left", vec!["left", "point"]),
                ("👉", "backhand index pointing right", vec!["right", "point"]),
                ("👆", "backhand index pointing up", vec!["up", "point"]),
                ("🖕", "middle finger", vec!["finger", "rude"]),
                ("👇", "backhand index pointing down", vec!["down", "point"]),
                ("☝️", "index pointing up", vec!["up", "point"]),
                ("👍", "thumbs up", vec!["like", "yes", "good"]),
                ("👎", "thumbs down", vec!["dislike", "no", "bad"]),
                ("✊", "raised fist", vec!["fist", "power"]),
                ("👊", "oncoming fist", vec!["punch", "fist"]),
                ("🤛", "left-facing fist", vec!["fist"]),
                ("🤜", "right-facing fist", vec!["fist"]),
                ("👏", "clapping hands", vec!["clap", "applause"]),
                ("🙌", "raising hands", vec!["celebrate", "hooray"]),
                ("👐", "open hands", vec!["hands"]),
                ("🤲", "palms up together", vec!["hands"]),
                ("🤝", "handshake", vec!["deal", "agreement"]),
                ("🙏", "folded hands", vec!["pray", "please", "thanks"]),
                ("✍️", "writing hand", vec!["write"]),
                ("💪", "flexed biceps", vec!["muscle", "strong"]),

                // Hearts
                ("❤️", "red heart", vec!["love", "heart"]),
                ("🧡", "orange heart", vec!["heart"]),
                ("💛", "yellow heart", vec!["heart"]),
                ("💚", "green heart", vec!["heart"]),
                ("💙", "blue heart", vec!["heart"]),
                ("💜", "purple heart", vec!["heart"]),
                ("🖤", "black heart", vec!["heart"]),
                ("🤍", "white heart", vec!["heart"]),
                ("🤎", "brown heart", vec!["heart"]),
                ("💔", "broken heart", vec!["heartbreak", "sad"]),
                ("💕", "two hearts", vec!["love", "hearts"]),
                ("💞", "revolving hearts", vec!["love", "hearts"]),
                ("💓", "beating heart", vec!["love", "heart"]),
                ("💗", "growing heart", vec!["love", "heart"]),
                ("💖", "sparkling heart", vec!["love", "heart"]),
                ("💘", "heart with arrow", vec!["love", "cupid"]),
                ("💝", "heart with ribbon", vec!["love", "gift"]),

                // Objects & Symbols
                ("🔥", "fire", vec!["hot", "lit", "flame"]),
                ("✨", "sparkles", vec!["magic", "shine"]),
                ("⭐", "star", vec!["star"]),
                ("🌟", "glowing star", vec!["star", "shine"]),
                ("💫", "dizzy", vec!["star", "dizzy"]),
                ("💯", "hundred points", vec!["100", "perfect"]),
                ("💢", "anger symbol", vec!["angry"]),
                ("💥", "collision", vec!["boom", "explosion"]),
                ("💦", "sweat droplets", vec!["water", "sweat"]),
                ("💨", "dashing away", vec!["fast", "wind"]),
                ("🕳️", "hole", vec!["hole"]),
                ("💣", "bomb", vec!["bomb"]),
                ("💬", "speech balloon", vec!["chat", "talk"]),
                ("👁️‍🗨️", "eye in speech bubble", vec!["witness"]),
                ("🗨️", "left speech bubble", vec!["talk"]),
                ("🗯️", "right anger bubble", vec!["angry"]),
                ("💭", "thought balloon", vec!["think"]),
                ("💤", "zzz", vec!["sleep", "tired"]),

                // Common symbols
                ("✅", "check mark button", vec!["check", "done", "yes"]),
                ("❌", "cross mark", vec!["no", "wrong", "x"]),
                ("❓", "question mark", vec!["question", "what"]),
                ("❗", "exclamation mark", vec!["exclamation", "important"]),
                ("⚠️", "warning", vec!["warning", "caution"]),
                ("🚫", "prohibited", vec!["no", "forbidden"]),
                ("⛔", "no entry", vec!["stop", "no"]),
                ("🔴", "red circle", vec!["red", "circle"]),
                ("🟠", "orange circle", vec!["orange", "circle"]),
                ("🟡", "yellow circle", vec!["yellow", "circle"]),
                ("🟢", "green circle", vec!["green", "circle"]),
                ("🔵", "blue circle", vec!["blue", "circle"]),
                ("🟣", "purple circle", vec!["purple", "circle"]),
                ("⚫", "black circle", vec!["black", "circle"]),
                ("⚪", "white circle", vec!["white", "circle"]),
            ],
        }
    }

    pub fn get_items(&self, query: &str) -> Vec<Item> {
        let query_lower = query.to_lowercase();

        self.emojis
            .iter()
            .filter(|(_, name, keywords)| {
                if query_lower.is_empty() {
                    return true;
                }
                name.contains(&query_lower)
                    || keywords.iter().any(|k| k.contains(&query_lower))
            })
            .map(|(emoji, name, _)| {
                Item::new(
                    format!("emoji:{}", emoji),
                    *emoji,
                    ItemType::Emoji,
                )
                .with_description(name.to_string())
            })
            .collect()
    }
}

impl Default for EmojiManager {
    fn default() -> Self {
        Self::new()
    }
}
