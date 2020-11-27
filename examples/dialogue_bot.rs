use umlsm::{
    CurrentStateIs, EntryVertex, ExitVertex, Guard, InitialPseudoState, ProcessEvent, ProcessResult,
};

// Vertexes

struct WaitForHello;
impl ExitVertex<NewMessage> for WaitForHello {
    fn exit(&mut self, _: &NewMessage) {}
}
impl EntryVertex<NewMessage> for WaitForHello {
    fn entry(&mut self, _: &NewMessage) {}
}
struct WaitForName;
impl ExitVertex<NewMessage> for WaitForName {
    fn exit(&mut self, _: &NewMessage) {}
}
impl EntryVertex<NewMessage> for WaitForName {
    fn entry(&mut self, _: &NewMessage) {}
}
struct WaitForAge {
    name: Option<String>,
}
impl EntryVertex<NewMessage> for WaitForAge {
    fn entry(&mut self, event: &NewMessage) {
        self.name = Some(event.0.clone())
    }
}
impl ExitVertex<NewMessage> for WaitForAge {
    fn exit(&mut self, _: &NewMessage) {
        self.name = None;
    }
}

// Guards

struct MesIs<'a>(&'a str);
impl Guard<NewMessage, String> for MesIs<'_> {
    fn check(&self, input: &NewMessage) -> Result<(), String> {
        match input.0.to_lowercase() == self.0 {
            true => Ok(()),
            false => Err("Please say `hello` for start of dialogue".to_string()),
        }
    }
}

fn is_number(mes: &NewMessage) -> Result<(), String> {
    match mes.0.chars().all(char::is_numeric) {
        true => Ok(()),
        false => Err("Please, put a number!".to_string()),
    }
}

// Events
struct NewMessage(String);

// Actions
fn start(_: &mut InitialPseudoState, _: &mut (), _: &()) -> String {
    "Hello! I am dialogue bot. Let's start! Say hello to me.".to_string()
}
fn hello(_: &mut WaitForHello, _: &mut (), _: &NewMessage) -> String {
    "Hello! How is your name?".to_string()
}
fn name(_: &mut WaitForName, _: &mut (), mes: &NewMessage) -> String {
    format!("Oh, your name is {}! How is your age?", mes.0)
}
fn age(state: &mut WaitForAge, _: &mut (), mes: &NewMessage) -> String {
    let age: u32 = mes.0.parse().unwrap();
    format!(
        "Oh, your name is {} and age is {}!",
        state.name.as_ref().unwrap(),
        age
    )
}

fn main() {
    #[rustfmt::skip]
    let mut sm = umlsm::state_machine!(
        state = (), err = String,
        [WaitForHello, WaitForName, WaitForAge { name: None }],

        InitialPseudoState + ()                          | start => WaitForHello,
        WaitForHello       + NewMessage [MesIs("hello")] | hello => WaitForName,
        WaitForName        + NewMessage [is_number]      | name  => WaitForAge,
        WaitForAge         + NewMessage                  | age   => WaitForHello
    );
    sm.process(&()).unwrap();
    assert!(sm.is::<WaitForHello>());

    repl("You > ", |input| {
        let mes = NewMessage(input);
        let answer = sm.process(&mes);
        match answer {
            ProcessResult::Handled(answer) => format!("Bot > {}", answer),
            ProcessResult::GuardErr(e) => format!("Bot > {}", e),
            _ => unreachable!(),
        }
    })
}

fn repl(s: &str, mut f: impl FnMut(String) -> String) {
    use std::io::Write;
    use std::{io, io::stdin};

    loop {
        print!("{}", s);
        io::stdout().flush().unwrap();
        let mut data = String::new();
        stdin().read_line(&mut data).expect("Error when read line");
        data.pop();
        match data.as_str() {
            "exit" => return,
            _ => println!("{}", f(data)),
        }
    }
}
