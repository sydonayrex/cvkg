use cvkg_macros::Form;

#[derive(Form)]
struct Signup {
    #[form(
        label = "Email",
        placeholder = "you@example.com",
        field_type = "email",
        required
    )]
    email: String,
    #[form(label = "Password", field_type = "password", min_len = 8)]
    password: String,
}

#[test]
fn derive_generates_field_descriptors() {
    let fields = Signup::form_fields();
    assert_eq!(fields.len(), 2);
    assert_eq!(fields[0].name, "email");
    assert_eq!(fields[0].label, "Email");
    assert_eq!(fields[0].field_type, "email");
    assert!(fields[0].required);
    assert_eq!(fields[1].min_length, 8);
}

#[test]
fn derive_generates_validation_calls() {
    let form = Signup {
        email: String::new(),
        password: "short".into(),
    };
    let errors = form.validate_form();
    assert!(errors.iter().any(|error| error.contains("email")));
    assert!(errors.iter().any(|error| error.contains("password")));
}
