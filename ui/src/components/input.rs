use iced::{
    Element, Length, Padding,
    widget::{Column, text, text_input},
};
use std::rc::Rc;

#[derive(Clone)]
pub enum ValidationRule {
    NonEmpty,
    IsUsize,
    Custom(Rc<dyn Fn(&str) -> Result<(), String>>),
}

impl std::fmt::Debug for ValidationRule {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationRule::NonEmpty => write!(f, "NonEmpty"),
            ValidationRule::IsUsize => write!(f, "IsUsize"),
            ValidationRule::Custom(_) => write!(f, "Custom(...)"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ValidatedInput {
    pub value: String,
    pub error: Option<String>,
    label: String,
    placeholder: String,
    rules: Vec<ValidationRule>,
    width: Option<Length>,
}

#[derive(Debug, Clone)]
pub enum ValidatedInputMessage {
    ValueChanged(String),
    Validate,
    Clear,
}

impl ValidatedInput {
    pub fn new(label: impl Into<String>, placeholder: impl Into<String>) -> Self {
        Self {
            value: String::new(),
            error: None,
            label: label.into(),
            placeholder: placeholder.into(),
            rules: Vec::new(),
            width: None,
        }
    }

    pub fn with_value(mut self, value: impl Into<String>) -> Self {
        self.value = value.into();
        self
    }

    pub fn with_rule(mut self, rule: ValidationRule) -> Self {
        self.rules.push(rule);
        self
    }

    pub fn with_rules(mut self, rules: Vec<ValidationRule>) -> Self {
        self.rules = rules;
        self
    }

    pub fn with_width(mut self, width: Length) -> Self {
        self.width = Some(width);
        self
    }

    pub fn non_empty(self) -> Self {
        self.with_rule(ValidationRule::NonEmpty)
    }

    pub fn as_usize(self) -> Self {
        self.with_rule(ValidationRule::IsUsize)
    }

    pub fn custom_validation<F>(self, validator: F) -> Self
    where
        F: Fn(&str) -> Result<(), String> + 'static,
    {
        self.with_rule(ValidationRule::Custom(Rc::new(validator)))
    }

    pub fn update(&mut self, message: ValidatedInputMessage) -> bool {
        match message {
            ValidatedInputMessage::ValueChanged(new_value) => {
                self.value = new_value;
                self.error = None;
                true
            }
            ValidatedInputMessage::Validate => {
                self.validate();
                false
            }
            ValidatedInputMessage::Clear => {
                self.value.clear();
                self.error = None;
                true
            }
        }
    }

    pub fn validate(&mut self) -> bool {
        self.error = None;

        for rule in &self.rules {
            if let Err(error_msg) = self.apply_rule(rule) {
                self.error = Some(error_msg);
                return false;
            }
        }
        true
    }

    fn apply_rule(&self, rule: &ValidationRule) -> Result<(), String> {
        match rule {
            ValidationRule::NonEmpty => {
                if self.value.trim().is_empty() {
                    Err("This field cannot be empty".to_string())
                } else {
                    Ok(())
                }
            }
            ValidationRule::IsUsize => {
                if self.value.trim().is_empty() {
                    Ok(())
                } else if self.value.parse::<usize>().is_err() {
                    Err("Must be a valid positive number".to_string())
                } else {
                    Ok(())
                }
            }
            ValidationRule::Custom(validator) => validator(&self.value),
        }
    }

    pub fn is_valid(&self) -> bool {
        self.error.is_none()
    }

    pub fn has_error(&self) -> bool {
        self.error.is_some()
    }

    pub fn get_usize_value(&self) -> Option<usize> {
        if self.value.trim().is_empty() {
            None
        } else {
            self.value.parse().ok()
        }
    }

    pub fn view<Message>(
        &self,
        font_size: f32,
        map_message: impl Fn(ValidatedInputMessage) -> Message + 'static,
    ) -> Element<'_, Message>
    where
        Message: 'static + Clone,
    {
        let mut input = text_input(&self.placeholder, &self.value)
            .padding(Padding {
                left: font_size,
                top: 0.75 * font_size,
                bottom: 0.75 * font_size,
                right: font_size,
            })
            .on_input(move |text| map_message(ValidatedInputMessage::ValueChanged(text)));

        if let Some(width) = self.width {
            input = input.width(width);
        }

        let mut elements = vec![text(&self.label).line_height(1.5).into(), input.into()];

        if let Some(error) = &self.error {
            elements.push(
                text(error)
                    .style(|_theme| iced::widget::text::Style {
                        color: Some(iced::Color::from_rgb(0.8, 0.2, 0.2)),
                    })
                    .size(font_size * 0.85)
                    .into(),
            );
        }

        Column::with_children(elements)
            .spacing(font_size * 0.25)
            .into()
    }

    // Convenience method for creating labeled inputs with consistent width
    pub fn view_with_label<Message>(
        &self,
        font_size: f32,
        label_width: f32,
        map_message: impl Fn(ValidatedInputMessage) -> Message + 'static,
    ) -> Element<'_, Message>
    where
        Message: 'static + Clone,
    {
        use iced::Alignment;
        use iced::widget::Row;

        let input = text_input(&self.placeholder, &self.value)
            .padding(Padding {
                left: font_size,
                top: 0.75 * font_size,
                bottom: 0.75 * font_size,
                right: font_size,
            })
            .on_input(move |text| map_message(ValidatedInputMessage::ValueChanged(text)));

        let main_row = Row::new()
            .push(text(&self.label).width(Length::Fixed(label_width)))
            .push(input)
            .align_y(Alignment::Center)
            .spacing(font_size * 0.5);

        // If there's an error, create a column with the row and error
        if let Some(error) = &self.error {
            Column::new()
                .push(main_row)
                .push(
                    text(error)
                        .style(|_theme| iced::widget::text::Style {
                            color: Some(iced::Color::from_rgb(0.8, 0.2, 0.2)),
                        })
                        .size(font_size * 0.85),
                )
                .spacing(font_size * 0.25)
                .into()
        } else {
            main_row.into()
        }
    }
}

// Helper macro for creating multiple validated inputs
#[macro_export]
macro_rules! validated_inputs {
    ($($name:ident: $label:expr, $placeholder:expr $(, $($method:ident$(($($arg:expr),*))?),*)?;)*) => {
        $(
            let mut $name = ValidatedInput::new($label, $placeholder);
            $($(
                $name = $name.$method($($($arg),*)?);
            )*)?
        )*
    };
}

// Example usage and helper functions
impl ValidatedInput {
    // Helper to create a directory input
    pub fn directory_input() -> Self {
        Self::new("Directory", "Directory for downloading the file").non_empty()
    }

    // Helper to create a URL input
    pub fn url_input() -> Self {
        Self::new("Link", "Enter your download link here")
            .non_empty()
            .custom_validation(|value| {
                if value.trim().is_empty() {
                    return Ok(());
                }
                if value.starts_with("http://") || value.starts_with("https://") {
                    Ok(())
                } else {
                    Err("Must be a valid URL starting with http:// or https://".to_string())
                }
            })
    }

    // Helper to create an optional text input
    pub fn optional_text_input(label: &str, placeholder: &str) -> Self {
        Self::new(label, placeholder)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validation() {
        let mut input = ValidatedInput::new("Test", "test").non_empty().as_usize();

        // Test empty value
        assert!(!input.validate());
        assert!(input.has_error());

        // Test non-numeric value
        input.value = "abc".to_string();
        assert!(!input.validate());

        // Test valid numeric value
        input.value = "123".to_string();
        assert!(input.validate());
        assert_eq!(input.get_usize_value(), Some(123));
    }
}
