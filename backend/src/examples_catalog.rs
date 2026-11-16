use crate::models::{CodeExample, SourceLanguage};

pub fn get_premade_examples() -> Vec<CodeExample> {
    vec![
        CodeExample {
            identifier: "cpp-hello-world".to_string(),
            title: "Hello World (C++)".to_string(),
            category: "Basics".to_string(),
            difficulty: "Easy".to_string(),
            source_language: SourceLanguage::Cpp,
            description: "Simple C++ program using std::string, int, and cout.".to_string(),
            source_code: r#"#include <iostream>
#include <string>
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
            title: "Fibonacci Sequence (C)".to_string(),
            category: "Algorithms".to_string(),
            difficulty: "Easy".to_string(),
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
            identifier: "cpp-float-vector".to_string(),
            title: "Float Vector Iteration (C++)".to_string(),
            category: "Vectors".to_string(),
            difficulty: "Easy".to_string(),
            source_language: SourceLanguage::Cpp,
            description: "Builds a std::vector<float> with push_back and prints items via range loop.".to_string(),
            source_code: r#"#include <iostream>
#include <vector>

int main() {
    std::vector<float> fvec;
    fvec.push_back(10.5);
    fvec.push_back(30.5);
    fvec.push_back(155.664);

    for (auto& vec : fvec) {
        std::cout << vec << std::endl;
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
            difficulty: "Medium".to_string(),
            source_language: SourceLanguage::Cpp,
            description: "Demonstrates std::vector and std::unordered_map translated to Vec and HashMap.".to_string(),
            source_code: r#"#include <iostream>
#include <vector>
#include <unordered_map>
#include <string>
using namespace std;

int main() {
    std::vector<int> numbers = {10, 20, 30};
    numbers.push_back(40);
    for (const auto& value : numbers) {
        cout << "Item: " << value << endl;
    }
    std::unordered_map<std::string, int> scores;
    scores.insert({"Alice", 95});
    scores["Bob"] = 88;
    cout << "Alice: " << scores["Alice"] << endl;
    if (scores.count("Bob")) {
        cout << "Has Bob: yes" << endl;
    }
    cout << "Count: " << numbers.size() << endl;
    return 0;
}"#
            .to_string(),
            default_stdin: String::new(),
        },
        CodeExample {
            identifier: "cpp-hashset-dedup".to_string(),
            title: "HashSet Deduplication & Sort (C++)".to_string(),
            category: "Data Structures".to_string(),
            difficulty: "Medium".to_string(),
            source_language: SourceLanguage::Cpp,
            description: "Uses std::unordered_set to deduplicate integers and std::sort to order them.".to_string(),
            source_code: r#"#include <iostream>
#include <vector>
#include <unordered_set>
#include <algorithm>

int main() {
    std::vector<int> rawValues = {42, 7, 19, 7, 42, 3, 19};
    std::unordered_set<int> seen;
    std::vector<int> uniqueSorted;

    for (const auto& value : rawValues) {
        if (!seen.count(value)) {
            seen.insert(value);
            uniqueSorted.push_back(value);
        }
    }

    std::sort(uniqueSorted.begin(), uniqueSorted.end());
    for (const auto& item : uniqueSorted) {
        std::cout << item << std::endl;
    }
    return 0;
}"#
            .to_string(),
            default_stdin: String::new(),
        },
        CodeExample {
            identifier: "c-stdin-io".to_string(),
            title: "Standard Input (C)".to_string(),
            category: "I/O".to_string(),
            difficulty: "Medium".to_string(),
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
        CodeExample {
            identifier: "cpp-class-struct".to_string(),
            title: "Class & Methods (C++)".to_string(),
            category: "OOP".to_string(),
            difficulty: "Hard".to_string(),
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
    int perimeter() {
        return 2 * (width + height);
    }
};

int main() {
    Rectangle rect;
    rect.width = 5;
    rect.height = 4;
    cout << "Area: " << rect.area() << endl;
    cout << "Perimeter: " << rect.perimeter() << endl;
    return 0;
}"#
            .to_string(),
            default_stdin: String::new(),
        },
        CodeExample {
            identifier: "cpp-binary-search-optional".to_string(),
            title: "Binary Search with Optional (C++)".to_string(),
            category: "Algorithms".to_string(),
            difficulty: "Hard".to_string(),
            source_language: SourceLanguage::Cpp,
            description: "Implements binary search returning std::optional<int> mapped to Rust Option<i32>.".to_string(),
            source_code: r#"#include <iostream>
#include <vector>
#include <optional>

std::optional<int> binarySearch(const std::vector<int>& nums, int target) {
    int left = 0;
    int right = nums.size() - 1;
    while (left <= right) {
        int mid = left + (right - left) / 2;
        if (nums[mid] == target) {
            return mid;
        } else if (nums[mid] < target) {
            left = mid + 1;
        } else {
            right = mid - 1;
        }
    }
    return std::nullopt;
}

int main() {
    std::vector<int> sortedNums = {2, 5, 8, 12, 16, 23, 38};
    std::optional<int> found = binarySearch(sortedNums, 16);
    if (found.has_value()) {
        std::cout << "Found at index: " << found.value() << std::endl;
    } else {
        std::cout << "Not found" << std::endl;
    }
    return 0;
}"#
            .to_string(),
            default_stdin: String::new(),
        },
        CodeExample {
            identifier: "cpp-template-pointer".to_string(),
            title: "Templates & Smart Pointers (C++)".to_string(),
            category: "Generics & Memory".to_string(),
            difficulty: "Hard".to_string(),
            source_language: SourceLanguage::Cpp,
            description: "Generic template class translated to Rust generics plus std::unique_ptr mapped to Box.".to_string(),
            source_code: r#"#include <iostream>
#include <memory>

template <typename T>
class ValueBox {
public:
    T item;
    T getItem() {
        return item;
    }
};

int main() {
    ValueBox<int> intBox;
    intBox.item = 42;
    std::unique_ptr<int> heapNumber = std::make_unique<int>(99);
    std::cout << "Box: " << intBox.getItem() << ", Heap: " << *heapNumber << std::endl;
    return 0;
}"#
            .to_string(),
            default_stdin: String::new(),
        },
        CodeExample {
            identifier: "cpp-two-sum-hashmap".to_string(),
            title: "Two Sum Solver (C++)".to_string(),
            category: "Algorithms".to_string(),
            difficulty: "Very Complex".to_string(),
            source_language: SourceLanguage::Cpp,
            description: "O(n) Two-Sum algorithm using std::unordered_map and std::vector.".to_string(),
            source_code: r#"#include <iostream>
#include <vector>
#include <unordered_map>

std::vector<int> twoSum(const std::vector<int>& nums, int target) {
    std::unordered_map<int, int> indexMap;
    for (int i = 0; i < nums.size(); i++) {
        int complement = target - nums[i];
        if (indexMap.count(complement)) {
            return {indexMap[complement], i};
        }
        indexMap[nums[i]] = i;
    }
    return {};
}

int main() {
    std::vector<int> nums = {2, 7, 11, 15};
    std::vector<int> result = twoSum(nums, 9);
    std::cout << result[0] << " " << result[1] << std::endl;
    return 0;
}"#
            .to_string(),
            default_stdin: String::new(),
        },
        CodeExample {
            identifier: "cpp-bfs-queue-graph".to_string(),
            title: "Graph BFS Traversal (C++)".to_string(),
            category: "Graphs".to_string(),
            difficulty: "Very Complex".to_string(),
            source_language: SourceLanguage::Cpp,
            description: "Breadth-first search using adjacency list std::vector<std::vector<int>> and std::queue<int>.".to_string(),
            source_code: r#"#include <iostream>
#include <vector>
#include <queue>

int main() {
    int nodeCount = 5;
    std::vector<std::vector<int>> adjacency(nodeCount);
    adjacency[0].push_back(1);
    adjacency[0].push_back(2);
    adjacency[1].push_back(3);
    adjacency[2].push_back(4);

    std::vector<bool> visited(nodeCount, false);
    std::queue<int> frontier;
    visited[0] = true;
    frontier.push(0);

    while (!frontier.empty()) {
        int current = frontier.front();
        frontier.pop();
        std::cout << "Visited: " << current << std::endl;
        for (int neighbor : adjacency[current]) {
            if (!visited[neighbor]) {
                visited[neighbor] = true;
                frontier.push(neighbor);
            }
        }
    }
    return 0;
}"#
            .to_string(),
            default_stdin: String::new(),
        },
        CodeExample {
            identifier: "cpp-dp-coin-change".to_string(),
            title: "Coin Change DP (C++)".to_string(),
            category: "Dynamic Programming".to_string(),
            difficulty: "Very Complex".to_string(),
            source_language: SourceLanguage::Cpp,
            description: "Bottom-up Dynamic Programming solving minimum coins for an amount using std::vector and std::min.".to_string(),
            source_code: r#"#include <iostream>
#include <vector>
#include <algorithm>

int coinChange(const std::vector<int>& coins, int amount) {
    int infinityValue = amount + 1;
    std::vector<int> dp(amount + 1, infinityValue);
    dp[0] = 0;

    for (int currentAmount = 1; currentAmount <= amount; currentAmount++) {
        for (int coin : coins) {
            if (coin <= currentAmount) {
                dp[currentAmount] = std::min(dp[currentAmount], dp[currentAmount - coin] + 1);
            }
        }
    }

    if (dp[amount] > amount) {
        return -1;
    }
    return dp[amount];
}

int main() {
    std::vector<int> coins = {1, 2, 5};
    int minimumCoins = coinChange(coins, 11);
    std::cout << "Min coins: " << minimumCoins << std::endl;
    return 0;
}"#
            .to_string(),
            default_stdin: String::new(),
        },
    ]
}
