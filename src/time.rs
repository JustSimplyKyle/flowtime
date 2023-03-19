#[derive(Default, Debug, Clone)]
pub struct Time {
    pub second: u16,
    pub minutes: u16,
    pub hour: u16,
}

impl Time {
    pub fn increment_second(&mut self) {
        self.second += 1;
        self.check_carry();
    }
    fn check_carry(&mut self) {
        if self.second == 59 {
            self.second = 0;
            self.minutes += 1;
        }
        if self.minutes == 59 {
            self.minutes = 0;
            self.hour += 1;
        }
        if self.hour == 23 {
            self.hour = 0;
        }
    }
    pub fn decrement_second(&mut self) {
        if self.second == 0 && self.minutes > 0 {
            self.second = 59;
            self.minutes -= 1;
        } else if self.minutes == 0 && self.hour > 0 {
            self.minutes = 59;
            self.hour -= 1;
        } else {
            self.second -= 1;
        }
    }
    pub fn formatted_string(&self) -> String {
        let mut output = String::new();
        if self.hour < 10 {
            output.push_str(&format!("0{}:", self.hour));
        } else {
            output.push_str(&format!("{}:", self.hour));
        }
        if self.minutes < 10 {
            output.push_str(format!("0{}:", self.minutes).as_str());
        } else {
            output.push_str(&format!("{}:", self.minutes));
        }
        if self.second < 10 {
            output.push_str(format!("0{}", self.second).as_str());
        } else {
            output.push_str(&format!("{}", self.second));
        }
        output
    }
    pub fn set_time_by_second(&mut self, seconds: u16) {
        self.hour = seconds / 3600;
        self.minutes = (seconds % 3600) / 60;
        self.second = seconds % 60;
    }
    pub fn get_second(&self) -> u16 {
        self.second + self.minutes * 60 + self.hour * 3600
    }
    pub fn reset_time(&mut self) {
        self.second = 0;
        self.minutes = 0;
        self.hour = 0;
    }
}
