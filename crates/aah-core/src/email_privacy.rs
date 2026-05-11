pub fn mask_email_address(email: &str) -> String {
    let Some((local_part, domain)) = email.split_once('@') else {
        return mask_middle(email);
    };

    format!("{}@{domain}", mask_middle(local_part))
}

pub fn display_email_address(email: &str, email_privacy_enabled: bool) -> String {
    if email_privacy_enabled {
        mask_email_address(email)
    } else {
        email.to_string()
    }
}

fn mask_middle(value: &str) -> String {
    let characters: Vec<char> = value.chars().collect();

    match characters.len() {
        0 | 1 => "***".to_string(),
        2 | 3 => format!("{}***", characters[0]),
        4 => format!("{}***{}", characters[0], characters[3]),
        length => {
            let prefix: String = characters.iter().take(2).collect();
            format!("{prefix}***{}", characters[length - 1])
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{display_email_address, mask_email_address};

    #[test]
    fn masks_middle_of_local_part_and_keeps_domain() {
        assert_eq!(
            mask_email_address("murong@example.com"),
            "mu***g@example.com"
        );
        assert_eq!(mask_email_address("me@example.com"), "m***@example.com");
        assert_eq!(mask_email_address("x@example.com"), "***@example.com");
    }

    #[test]
    fn display_email_uses_raw_value_when_privacy_is_disabled() {
        assert_eq!(
            display_email_address("murong@example.com", false),
            "murong@example.com"
        );
        assert_eq!(
            display_email_address("murong@example.com", true),
            "mu***g@example.com"
        );
    }
}
