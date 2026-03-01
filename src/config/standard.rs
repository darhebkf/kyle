use super::kylefile::{Kylefile, Task};
use std::collections::HashMap;

fn make_task(cmd: &str, desc: &str) -> Task {
    Task {
        desc: desc.to_string(),
        run: cmd.to_string(),
        ..Default::default()
    }
}

fn build_kylefile(name: &str, entries: &[(&str, &str, &str)]) -> Kylefile {
    let mut tasks = HashMap::new();
    for (task_name, cmd, desc) in entries {
        tasks.insert(task_name.to_string(), make_task(cmd, desc));
    }
    Kylefile {
        name: name.to_string(),
        tasks,
        ..Default::default()
    }
}

pub fn cargo() -> Kylefile {
    build_kylefile(
        "",
        &[
            ("build", "cargo build", "Build the project"),
            ("test", "cargo test", "Run tests"),
            ("run", "cargo run", "Run the project"),
            ("check", "cargo check", "Check for errors"),
            ("clippy", "cargo clippy", "Run linter"),
            ("fmt", "cargo fmt", "Format code"),
        ],
    )
}

pub fn go_mod() -> Kylefile {
    build_kylefile(
        "",
        &[
            ("build", "go build ./...", "Build packages"),
            ("test", "go test ./...", "Run tests"),
            ("run", "go run .", "Run the project"),
            ("vet", "go vet ./...", "Run vet"),
            ("fmt", "gofmt -w .", "Format code"),
        ],
    )
}

pub fn pubspec() -> Kylefile {
    build_kylefile(
        "",
        &[
            ("run", "flutter run", "Run the app"),
            ("build", "flutter build", "Build the app"),
            ("test", "flutter test", "Run tests"),
            ("analyze", "flutter analyze", "Analyze code"),
            ("pub-get", "flutter pub get", "Get dependencies"),
        ],
    )
}

pub fn dotnet() -> Kylefile {
    build_kylefile(
        "",
        &[
            ("build", "dotnet build", "Build the project"),
            ("test", "dotnet test", "Run tests"),
            ("run", "dotnet run", "Run the project"),
            ("publish", "dotnet publish", "Publish the project"),
            ("clean", "dotnet clean", "Clean build output"),
        ],
    )
}

pub fn gradle() -> Kylefile {
    build_kylefile(
        "",
        &[
            ("build", "gradle build", "Build the project"),
            ("test", "gradle test", "Run tests"),
            ("run", "gradle run", "Run the project"),
            ("clean", "gradle clean", "Clean build output"),
        ],
    )
}

pub fn maven() -> Kylefile {
    build_kylefile(
        "",
        &[
            ("compile", "mvn compile", "Compile sources"),
            ("test", "mvn test", "Run tests"),
            ("package", "mvn package", "Package the project"),
            ("install", "mvn install", "Install to local repo"),
            ("clean", "mvn clean", "Clean build output"),
        ],
    )
}

pub fn cmake() -> Kylefile {
    build_kylefile(
        "",
        &[
            ("configure", "cmake -B build", "Configure the build"),
            ("build", "cmake --build build", "Build the project"),
            ("test", "cd build && ctest", "Run tests"),
            ("clean", "rm -rf build", "Clean build output"),
        ],
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cargo_has_standard_tasks() {
        let kf = cargo();
        assert!(kf.tasks.contains_key("build"));
        assert!(kf.tasks.contains_key("test"));
        assert!(kf.tasks.contains_key("run"));
        assert!(kf.tasks.contains_key("clippy"));
        assert!(kf.tasks.contains_key("fmt"));
        assert_eq!(kf.tasks["build"].run, "cargo build");
    }

    #[test]
    fn go_has_standard_tasks() {
        let kf = go_mod();
        assert!(kf.tasks.contains_key("build"));
        assert!(kf.tasks.contains_key("test"));
        assert!(kf.tasks.contains_key("vet"));
        assert_eq!(kf.tasks["build"].run, "go build ./...");
    }

    #[test]
    fn dotnet_has_standard_tasks() {
        let kf = dotnet();
        assert!(kf.tasks.contains_key("build"));
        assert!(kf.tasks.contains_key("test"));
        assert!(kf.tasks.contains_key("run"));
        assert_eq!(kf.tasks["build"].run, "dotnet build");
    }

    #[test]
    fn all_generators_return_non_empty() {
        assert!(!cargo().tasks.is_empty());
        assert!(!go_mod().tasks.is_empty());
        assert!(!pubspec().tasks.is_empty());
        assert!(!dotnet().tasks.is_empty());
        assert!(!gradle().tasks.is_empty());
        assert!(!maven().tasks.is_empty());
        assert!(!cmake().tasks.is_empty());
    }
}
