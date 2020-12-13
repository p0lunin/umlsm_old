use umlsm::{
    Action, CurrentStateIs, EntryVertex, ExitVertex, Guard, InitialPseudoState, ProcessEvent,
    ProcessResult, TerminationPseudoState,
};

// Vertexes

struct WaitForHello;
impl EntryVertex for WaitForHello {}
impl ExitVertex for WaitForHello {}
struct WaitForName;
impl ExitVertex for WaitForName {}
impl EntryVertex for WaitForName {}
#[derive(Debug)]
struct WaitForAge {
    name: Option<String>,
}
impl EntryVertex for WaitForAge {}
impl ExitVertex for WaitForAge {
    fn exit(&mut self) {
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
struct Exit;

// Actions
fn start(_: &mut InitialPseudoState, _: &mut (), _: &(), _: &mut WaitForHello) -> String {
    "Hello! I am dialogue bot. Let's start! Say hello to me.".to_string()
}
fn hello(_: &mut WaitForHello, _: &mut (), _: &NewMessage, _: &mut WaitForName) -> String {
    "Hello! How is your name?".to_string()
}
fn name(_: &mut WaitForName, _: &mut (), mes: &NewMessage, age: &mut WaitForAge) -> String {
    age.name = Some(mes.0.clone());
    format!("Oh, your name is {}! How is your age?", mes.0)
}
fn age(state: &mut WaitForAge, _: &mut (), mes: &NewMessage, _: &mut WaitForHello) -> String {
    let age: u32 = mes.0.parse().unwrap();
    format!(
        "Oh, your name is {} and age is {}!",
        state.name.as_ref().unwrap(),
        age
    )
}

#[derive(Clone)]
struct ExitAction;
impl<Source> Action<Source, (), Exit, TerminationPseudoState, String> for ExitAction {
    fn trigger(
        &self,
        _: &mut Source,
        _: &mut (),
        _: &Exit,
        _: &mut TerminationPseudoState,
    ) -> String {
        "Bye, Bye!".to_string()
    }
}

fn main() {
    #[rustfmt::skip]
    let mut sm = umlsm::state_machine!(
        state = (), err = String,
        [WaitForHello, WaitForName, WaitForAge { name: None }],

        @InitialPseudoState + ()                          | start       => WaitForHello,
        @WaitForHello       + NewMessage [MesIs("hello")] | hello       => WaitForName,
        @WaitForName        + NewMessage                  | name        => WaitForAge,
        @WaitForAge         + NewMessage [is_number]      | age         => WaitForHello,

        forall:             + Exit                        | ExitAction  => TerminationPseudoState,
    );
    let mes = sm.process(&()).unwrap();
    println!("{}", mes);
    assert!(sm.is::<WaitForHello>());

    repl("You > ", |input| {
        let answer = match input.as_str() {
            "exit" => sm.process(&Exit),
            _ => sm.process(&NewMessage(input)),
        };
        match answer {
            ProcessResult::Handled(answer) => {
                if sm.is::<TerminationPseudoState>() {
                    Err(format!("Bot > {}", answer))
                } else {
                    Ok(format!("Bot > {}", answer))
                }
            }
            ProcessResult::GuardErr(e) => Ok(format!("Bot > {}", e)),
            _ => unreachable!(),
        }
    })
}

fn repl(s: &str, mut f: impl FnMut(String) -> Result<String, String>) {
    use std::io::Write;
    use std::{io, io::stdin};

    loop {
        print!("{}", s);
        io::stdout().flush().unwrap();
        let mut data = String::new();
        stdin().read_line(&mut data).expect("Error when read line");
        data.pop();
        match f(data) {
            Ok(d) => println!("{}", d),
            Err(text) => {
                println!("{}", text);
                return;
            }
        }
    }
}
