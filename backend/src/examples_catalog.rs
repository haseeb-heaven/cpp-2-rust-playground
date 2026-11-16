use crate::models::{CodeExample, SourceLanguage};

pub fn get_premade_examples() -> Vec<CodeExample> {
    vec![
        CodeExample {
            identifier: "cpp-hello-world".to_string(),
            title: "Hello World (C++)".to_string(),
            category: "Basics".to_string(),
            source_language: SourceLanguage::Cpp,
            description: "Simple C++ program using cout and variables.".to_string(),
            source_code: r#"#include <iostream>
using namespace std;

int main() {
    string studioName = "RustC-Playground";
    int version = 1;
    cout << "Welcome to " << studioName << " v" << version << endl;
    return 0;
}"#
            .to_string(),
            default_stdin: String::new(),
        },
        CodeExample {
            identifier: "c-loops-printf".to_string(),
            title: "Fibonacci & Loops (C)".to_string(),
            category: "Algorithms".to_string(),
            source_language: SourceLanguage::C,
            description: "C program computing Fibonacci numbers with a for loop and printf.".to_string(),
            source_code: r#"#include <stdio.h>

int main() {
    int first = 0;
    int second = 1;
    int limit = 8;
    for (int index = 0; index < limit; index++) {
        printf("%d\n", first);
        int next = first + second;
        first = second;
        second = next;
    }
    return 0;
}"#
            .to_string(),
            default_stdin: String::new(),
        },
        CodeExample {
            identifier: "cpp-collections".to_string(),
            title: "Vector & HashMap (C++)".to_string(),
            category: "Data Structures".to_string(),
            source_language: SourceLanguage::Cpp,
            description: "Demonstrates std::vector and std::unordered_map translated to Vec and HashMap.".to_string(),
            source_code: r#"#include <iostream>
#include <vector>
#include <unordered_map>
using namespace std;

int main() {
    std::vector<int> numbers = {10, 20, 30};
    numbers.push_back(40);
    for (const auto& value : numbers) {
        cout << "Item: " << value << endl;
    }
    std::unordered_map<std::string, int> scores;
    scores.insert({"Alice", 95});
    cout << "Count: " << numbers.size() << endl;
    return 0;
}"#
            .to_string(),
            default_stdin: String::new(),
        },
        CodeExample {
            identifier: "cpp-class-struct".to_string(),
            title: "Class & Methods (C++)".to_string(),
            category: "OOP".to_string(),
            source_language: SourceLanguage::Cpp,
            description: "Translates a C++ class with fields and methods into a Rust struct and impl block.".to_string(),
            source_code: r#"#include <iostream>
using namespace std;

class Rectangle {
public:
    int width;
    int height;
    int area() {
        return width * height;
    }
};

int main() {
    Rectangle rect = Rectangle { width: 5, height: 4 };
    cout << "Area: " << rect.area() << endl;
    return 0;
}"#
            .to_string(),
            default_stdin: String::new(),
        },
        CodeExample {
            identifier: "c-stdin-io".to_string(),
            title: "Standard Input (C)".to_string(),
            category: "I/O".to_string(),
            source_language: SourceLanguage::C,
            description: "Reads an integer from stdin using scanf and prints its square.".to_string(),
            source_code: r#"#include <stdio.h>

int main() {
    int number = 0;
    scanf("%d", &number);
    int squared = number * number;
    printf("Squared: %d\n", squared);
    return 0;
}"#
            .to_string(),
            default_stdin: "7\n".to_string(),
        },
    ]
}
