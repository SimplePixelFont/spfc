// Adapted from https://github.com/dansanderson/picotool/blob/main/pico8/lua/lua.py#L106-L289

#[derive(Copy, Clone, Debug)]
pub struct P8Char {
    pub id: u8,
    pub symbol: &'static str,
    pub description: &'static str,
}

const ASCII_POOL: &str = "!\"#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstuvwxyz{|}~";

const fn generate_character_set() -> [P8Char; 256] {
    let mut character_set = [P8Char { id: 0, symbol: "", description: "" }; 256];
    // Control codes
    character_set[0] = P8Char { id: 0, symbol: "\x00", description: "Terminate printing" };
    character_set[1] = P8Char { id: 1, symbol: "\x01", description: "Repeat next character P0 times" };
    character_set[2] = P8Char { id: 2, symbol: "\x02", description: "Draw solid background with color P0" };
    character_set[3] = P8Char { id: 3, symbol: "\x03", description: "Move cursor horizontally by P0-16 pixels" };
    character_set[4] = P8Char { id: 4, symbol: "\x04", description: "Move cursor vertically by P0-16 pixels" };
    character_set[5] = P8Char { id: 5, symbol: "\x05", description: "Move cursor by P0-16, P1-16 pixels" };
    character_set[6] = P8Char { id: 6, symbol: "\x06", description: "Special command" };
    character_set[7] = P8Char { id: 7, symbol: "\x07", description: "Audio command" };
    character_set[8] = P8Char { id: 8, symbol: "\x08", description: "Backspace" };
    character_set[9] = P8Char { id: 9, symbol: "\x09", description: "Tab" };
    character_set[10] = P8Char { id: 10, symbol: "\x0a", description: "Newline" };
    character_set[11] = P8Char { id: 11, symbol: "\x0b", description: "Decorate previous character command" };
    character_set[12] = P8Char { id: 12, symbol: "\x0c", description: "Set foreground to color P0" };
    character_set[13] = P8Char { id: 13, symbol: "\x0d", description: "Ccharacter_setiage return" };
    character_set[14] = P8Char { id: 14, symbol: "\x0e", description: "Switch font defined at 0x5600" };
    character_set[15] = P8Char { id: 15, symbol: "\x0f", description: "Switch font to default" };

    // Japanese punctuation
    character_set[16] = P8Char { id: 16, symbol: "▮", description: "Vertical rectangle" };
    character_set[17] = P8Char { id: 17, symbol: "■", description: "Filled square" };
    character_set[18] = P8Char { id: 18, symbol: "□", description: "Hollow square" };
    character_set[19] = P8Char { id: 19, symbol: "⁙", description: "Five dot" };
    character_set[20] = P8Char { id: 20, symbol: "⁘", description: "Four dot" };
    character_set[21] = P8Char { id: 21, symbol: "‖", description: "Pause" };
    character_set[22] = P8Char { id: 22, symbol: "◀", description: "Back" };
    character_set[23] = P8Char { id: 23, symbol: "▶", description: "Forward" };
    character_set[24] = P8Char { id: 24, symbol: "「", description: "Japanese starting quote" };
    character_set[25] = P8Char { id: 25, symbol: "」", description: "Japanese ending quote" };
    character_set[26] = P8Char { id: 26, symbol: "¥", description: "Yen sign" };
    character_set[27] = P8Char { id: 27, symbol: "•", description: "Interpunct" };
    character_set[28] = P8Char { id: 28, symbol: "、", description: "Japanese comma" };
    character_set[29] = P8Char { id: 29, symbol: "。", description: "Japanese full stop" };
    character_set[30] = P8Char { id: 30, symbol: "゛", description: "Japanese dakuten" };
    character_set[31] = P8Char { id: 31, symbol: "゜", description: "Japanese handakuten" };

    // ASCII
    character_set[32] = P8Char { id: 32, symbol: " ", description: "space" };

    let mut i: usize = 33;
    while i <= 126 {
        let pool_index = i - 33;
        // Unsafe is "safe" since we are using a predefined string that will always work :)
        let symbol = unsafe {
            std::str::from_utf8_unchecked(std::slice::from_raw_parts(
                ASCII_POOL.as_ptr().add(pool_index), 
                1
            ))
        };
        character_set[i] = P8Char { id: i as u8, symbol: symbol, description: symbol };
        i += 1;
    }

    character_set[127] = P8Char { id: 127, symbol: "○", description: "Hollow circle" };

    // Symbols
    character_set[128] = P8Char { id: 128, symbol: "█", description: "Rectangle" };
    character_set[129] = P8Char { id: 129, symbol: "▒", description: "Checkerboard" };
    character_set[130] = P8Char { id: 130, symbol: "🐱", description: "Jelpi" };
    character_set[131] = P8Char { id: 131, symbol: "⬇️", description: "Down key" };
    character_set[132] = P8Char { id: 132, symbol: "░", description: "Dot pattern" };
    character_set[133] = P8Char { id: 133, symbol: "✽", description: "Throwing star" };
    character_set[134] = P8Char { id: 134, symbol: "●", description: "Ball" };
    character_set[135] = P8Char { id: 135, symbol: "♥", description: "Heart" };
    character_set[136] = P8Char { id: 136, symbol: "☉", description: "Eye" };
    character_set[137] = P8Char { id: 137, symbol: "웃", description: "Man" };
    character_set[138] = P8Char { id: 138, symbol: "⌂", description: "House" };
    character_set[139] = P8Char { id: 139, symbol: "⬅️", description: "Left key" };
    character_set[140] = P8Char { id: 140, symbol: "😐", description: "Face" };
    character_set[141] = P8Char { id: 141, symbol: "♪", description: "Musical note" };
    character_set[142] = P8Char { id: 142, symbol: "🅾️", description: "O key" };
    character_set[143] = P8Char { id: 143, symbol: "◆", description: "Diamond" };
    character_set[144] = P8Char { id: 144, symbol: "…", description: "Ellipsis" };
    character_set[145] = P8Char { id: 145, symbol: "➡️", description: "Right key" };
    character_set[146] = P8Char { id: 146, symbol: "★", description: "Five-pointed star" };
    character_set[147] = P8Char { id: 147, symbol: "⧗", description: "Hourglass" };
    character_set[148] = P8Char { id: 148, symbol: "⬆️", description: "Up key" };
    character_set[149] = P8Char { id: 149, symbol: "ˇ", description: "Birds" };
    character_set[150] = P8Char { id: 150, symbol: "∧", description: "Sawtooth" };
    character_set[151] = P8Char { id: 151, symbol: "❎", description: "X key" };
    character_set[152] = P8Char { id: 152, symbol: "▤", description: "Horiz lines" };
    character_set[153] = P8Char { id: 153, symbol: "▥", description: "Vert lines" };

    // Hiragana
    character_set[154] = P8Char { id: 154, symbol: "あ", description: "Hiragana: a" };
    character_set[155] = P8Char { id: 155, symbol: "い", description: "i" };
    character_set[156] = P8Char { id: 156, symbol: "う", description: "u" };
    character_set[157] = P8Char { id: 157, symbol: "え", description: "e" };
    character_set[158] = P8Char { id: 158, symbol: "お", description: "o" };
    character_set[159] = P8Char { id: 159, symbol: "か", description: "ka" };
    character_set[160] = P8Char { id: 160, symbol: "き", description: "ki" };
    character_set[161] = P8Char { id: 161, symbol: "く", description: "ku" };
    character_set[162] = P8Char { id: 162, symbol: "け", description: "ke" };
    character_set[163] = P8Char { id: 163, symbol: "こ", description: "ko" };
    character_set[164] = P8Char { id: 164, symbol: "さ", description: "sa" };
    character_set[165] = P8Char { id: 165, symbol: "し", description: "si" };
    character_set[166] = P8Char { id: 166, symbol: "す", description: "su" };
    character_set[167] = P8Char { id: 167, symbol: "せ", description: "se" };
    character_set[168] = P8Char { id: 168, symbol: "そ", description: "so" };
    character_set[169] = P8Char { id: 169, symbol: "た", description: "ta" };
    character_set[170] = P8Char { id: 170, symbol: "ち", description: "chi" };
    character_set[171] = P8Char { id: 171, symbol: "つ", description: "tsu" };
    character_set[172] = P8Char { id: 172, symbol: "て", description: "te" };
    character_set[173] = P8Char { id: 173, symbol: "と", description: "to" };
    character_set[174] = P8Char { id: 174, symbol: "な", description: "na" };
    character_set[175] = P8Char { id: 175, symbol: "に", description: "ni" };
    character_set[176] = P8Char { id: 176, symbol: "ぬ", description: "nu" };
    character_set[177] = P8Char { id: 177, symbol: "ね", description: "ne" };
    character_set[178] = P8Char { id: 178, symbol: "の", description: "no" };
    character_set[179] = P8Char { id: 179, symbol: "は", description: "ha" };
    character_set[180] = P8Char { id: 180, symbol: "ひ", description: "hi" };
    character_set[181] = P8Char { id: 181, symbol: "ふ", description: "phu" };
    character_set[182] = P8Char { id: 182, symbol: "へ", description: "he" };
    character_set[183] = P8Char { id: 183, symbol: "ほ", description: "ho" };
    character_set[184] = P8Char { id: 184, symbol: "ま", description: "ma" };
    character_set[185] = P8Char { id: 185, symbol: "み", description: "mi" };
    character_set[186] = P8Char { id: 186, symbol: "む", description: "mu" };
    character_set[187] = P8Char { id: 187, symbol: "め", description: "me" };
    character_set[188] = P8Char { id: 188, symbol: "も", description: "mo" };
    character_set[189] = P8Char { id: 189, symbol: "や", description: "ya" };
    character_set[190] = P8Char { id: 190, symbol: "ゆ", description: "yu" };
    character_set[191] = P8Char { id: 191, symbol: "よ", description: "yo" };
    character_set[192] = P8Char { id: 192, symbol: "ら", description: "ra" };
    character_set[193] = P8Char { id: 193, symbol: "り", description: "ri" };
    character_set[194] = P8Char { id: 194, symbol: "る", description: "ru" };
    character_set[195] = P8Char { id: 195, symbol: "れ", description: "re" };
    character_set[196] = P8Char { id: 196, symbol: "ろ", description: "ro" };
    character_set[197] = P8Char { id: 197, symbol: "わ", description: "wa" };
    character_set[198] = P8Char { id: 198, symbol: "を", description: "wo" };
    character_set[199] = P8Char { id: 199, symbol: "ん", description: "n" };
    character_set[200] = P8Char { id: 200, symbol: "っ", description: "Hiragana sokuon" };
    character_set[201] = P8Char { id: 201, symbol: "ゃ", description: "Hiragana digraphs: ya" };
    character_set[202] = P8Char { id: 202, symbol: "ゅ", description: "yu" };
    character_set[203] = P8Char { id: 203, symbol: "ょ", description: "yo" };

    // Katakana
    character_set[204] = P8Char { id: 204, symbol: "ア", description: "Katakana: a" };
    character_set[205] = P8Char { id: 205, symbol: "イ", description: "i" };
    character_set[206] = P8Char { id: 206, symbol: "ウ", description: "u" };
    character_set[207] = P8Char { id: 207, symbol: "エ", description: "e" };
    character_set[208] = P8Char { id: 208, symbol: "オ", description: "o" };
    character_set[209] = P8Char { id: 209, symbol: "カ", description: "ka" };
    character_set[210] = P8Char { id: 210, symbol: "キ", description: "ki" };
    character_set[211] = P8Char { id: 211, symbol: "ク", description: "ku" };
    character_set[212] = P8Char { id: 212, symbol: "ケ", description: "ke" };
    character_set[213] = P8Char { id: 213, symbol: "コ", description: "ko" };
    character_set[214] = P8Char { id: 214, symbol: "サ", description: "sa" };
    character_set[215] = P8Char { id: 215, symbol: "シ", description: "si" };
    character_set[216] = P8Char { id: 216, symbol: "ス", description: "su" };
    character_set[217] = P8Char { id: 217, symbol: "セ", description: "se" };
    character_set[218] = P8Char { id: 218, symbol: "ソ", description: "so" };
    character_set[219] = P8Char { id: 219, symbol: "タ", description: "ta" };
    character_set[220] = P8Char { id: 220, symbol: "チ", description: "chi" };
    character_set[221] = P8Char { id: 221, symbol: "ツ", description: "tsu" };
    character_set[222] = P8Char { id: 222, symbol: "テ", description: "te" };
    character_set[223] = P8Char { id: 223, symbol: "ト", description: "to" };
    character_set[224] = P8Char { id: 224, symbol: "ナ", description: "na" };
    character_set[225] = P8Char { id: 225, symbol: "ニ", description: "ni" };
    character_set[226] = P8Char { id: 226, symbol: "ヌ", description: "nu" };
    character_set[227] = P8Char { id: 227, symbol: "ネ", description: "ne" };
    character_set[228] = P8Char { id: 228, symbol: "ノ", description: "no" };
    character_set[229] = P8Char { id: 229, symbol: "ハ", description: "ha" };
    character_set[230] = P8Char { id: 230, symbol: "ヒ", description: "hi" };
    character_set[231] = P8Char { id: 231, symbol: "フ", description: "phu" };
    character_set[232] = P8Char { id: 232, symbol: "ヘ", description: "he" };
    character_set[233] = P8Char { id: 233, symbol: "ホ", description: "ho" };
    character_set[234] = P8Char { id: 234, symbol: "マ", description: "ma" };
    character_set[235] = P8Char { id: 235, symbol: "ミ", description: "mi" };
    character_set[236] = P8Char { id: 236, symbol: "ム", description: "mu" };
    character_set[237] = P8Char { id: 237, symbol: "メ", description: "me" };
    character_set[238] = P8Char { id: 238, symbol: "モ", description: "mo" };
    character_set[239] = P8Char { id: 239, symbol: "ヤ", description: "ya" };
    character_set[240] = P8Char { id: 240, symbol: "ユ", description: "yu" };
    character_set[241] = P8Char { id: 241, symbol: "ヨ", description: "yo" };
    character_set[242] = P8Char { id: 242, symbol: "ラ", description: "ra" };
    character_set[243] = P8Char { id: 243, symbol: "リ", description: "ri" };
    character_set[244] = P8Char { id: 244, symbol: "ル", description: "ru" };
    character_set[245] = P8Char { id: 245, symbol: "レ", description: "re" };
    character_set[246] = P8Char { id: 246, symbol: "ロ", description: "ro" };
    character_set[247] = P8Char { id: 247, symbol: "ワ", description: "wa" };
    character_set[248] = P8Char { id: 248, symbol: "ヲ", description: "wo" };
    character_set[249] = P8Char { id: 249, symbol: "ン", description: "n" };
    character_set[250] = P8Char { id: 250, symbol: "ッ", description: "Katakana sokuon" };
    character_set[251] = P8Char { id: 251, symbol: "ャ", description: "Katakana digraphs: ya" };
    character_set[252] = P8Char { id: 252, symbol: "ュ", description: "yu" };
    character_set[253] = P8Char { id: 253, symbol: "ョ", description: "yo" };

    // Remaining symbols
    character_set[254] = P8Char { id: 254, symbol: "◜", description: "Left arc" };
    character_set[255] = P8Char { id: 255, symbol: "◝", description: "Right arc" };

    character_set
}

pub const P8SCII_CHARSET: [P8Char; 256] = generate_character_set();