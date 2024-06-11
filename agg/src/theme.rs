use anyhow::{bail, Error, Result};
use clap::ValueEnum;
use rgb::RGB8;
use std::str::FromStr;

#[derive(Debug, Clone, Default, ValueEnum)]
pub enum ThemeName {
    Asciinema,
    #[default]
    Dracula,
    GithubDark,
    GithubLight,
    Monokai,
    Nord,
    SolarizedDark,
    SolarizedLight,
}

impl ThemeName {
    pub fn get_theme(&self) -> Result<Theme> {
        match self {
            Self::Asciinema => "121314,cccccc,000000,dd3c69,4ebf22,ddaf3c,26b0d7,b954e1,54e1b9,d9d9d9,4d4d4d,dd3c69,4ebf22,ddaf3c,26b0d7,b954e1,54e1b9,ffffff".parse(),
            Self::Dracula => "282a36,f8f8f2,21222c,ff5555,50fa7b,f1fa8c,bd93f9,ff79c6,8be9fd,f8f8f2,6272a4,ff6e6e,69ff94,ffffa5,d6acff,ff92df,a4ffff,ffffff".parse(),
            Self::GithubDark => "171b21,eceff4,0e1116,f97583,a2fca2,fabb72,7db4f9,c4a0f5,1f6feb,eceff4,6a737d,bf5a64,7abf7a,bf8f57,608bbf,997dbf,195cbf,b9bbbf".parse(),
            Self::GithubLight => "eceff4,171b21,0e1116,f97583,a2fca2,fabb72,7db4f9,c4a0f5,1f6feb,eceff4,6a737d,bf5a64,7abf7a,bf8f57,608bbf,997dbf,195cbf,b9bbbf".parse(),
            Self::Monokai => "272822,f8f8f2,272822,f92672,a6e22e,f4bf75,66d9ef,ae81ff,a1efe4,f8f8f2,75715e,f92672,a6e22e,f4bf75,66d9ef,ae81ff,a1efe4,f9f8f5".parse(),
            Self::Nord => "2e3440,eceff4,3b4252,bf616a,a3be8c,ebcb8b,81a1c1,b48ead,88c0d0,eceff4,3b4252,bf616a,a3be8c,ebcb8b,81a1c1,b48ead,88c0d0,eceff4".parse(),
            Self::SolarizedDark => "002b36,839496,073642,dc322f,859900,b58900,268bd2,d33682,2aa198,eee8d5,002b36,cb4b16,586e75,657b83,839496,6c71c4,93a1a1,fdf6e3".parse(),
            Self::SolarizedLight => "fdf6e3,657b83,073642,dc322f,859900,b58900,268bd2,d33682,2aa198,eee8d5,002b36,cb4b16,586e75,657c83,839496,6c71c4,93a1a1,fdf6e3".parse(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Theme {
    pub background: RGB8,
    pub foreground: RGB8,
    palette: [RGB8; 16],
}

fn parse_hex_triplet(triplet: &str) -> Result<RGB8> {
    if triplet.len() == 6 {
        let r = u8::from_str_radix(&triplet[0..2], 16)?;
        let g = u8::from_str_radix(&triplet[2..4], 16)?;
        let b = u8::from_str_radix(&triplet[4..6], 16)?;
        Ok(RGB8::new(r, g, b))
    } else {
        bail!("{} is not a hex triplet", triplet)
    }
}

impl FromStr for Theme {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let mut palette = [RGB8::default(); 16];
        let colors = s
            .split(',')
            .filter(|s| !s.is_empty())
            .map(parse_hex_triplet)
            .collect::<Result<Vec<RGB8>>>()?;
        if colors.len() != 10 && colors.len() != 18 {
            bail!("expected 10 or 18 hex triplets, got {}", colors.len());
        }
        let background = colors[0];
        let foreground = colors[1];
        for (i, color) in colors.into_iter().skip(2).cycle().take(16).enumerate() {
            palette[i] = color;
        }
        Ok(Self {
            background,
            foreground,
            palette,
        })
    }
}

impl Theme {
    pub fn color(&self, color: u8) -> RGB8 {
        match color {
            0..=15 => self.palette[color as usize],
            16..=231 => {
                let n = color - 16;
                let mut r = ((n / 36) % 6) * 40;
                let mut g = ((n / 6) % 6) * 40;
                let mut b = (n % 6) * 40;
                if r > 0 {
                    r += 55;
                }
                if g > 0 {
                    g += 55;
                }
                if b > 0 {
                    b += 55;
                }
                RGB8::new(r, g, b)
            }
            232.. => {
                let v = 8 + 10 * (color - 232);
                RGB8::new(v, v, v)
            }
        }
    }
}
