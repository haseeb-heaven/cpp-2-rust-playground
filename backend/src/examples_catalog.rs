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
        CodeExample {
            identifier: "c-base-validator".to_string(),
            title: "Number Base Validator (C)".to_string(),
            category: "String & Validation".to_string(),
            difficulty: "Hard".to_string(),
            source_language: SourceLanguage::C,
            description: "Uses C forward declarations, fgets/strcspn/memmove input parsing, static const char* arrays, and pointer iteration.".to_string(),
            source_code: r#"#include <stdio.h>
#include <ctype.h>
#include <stdbool.h>
#include <stdlib.h>
#include <string.h>

bool IsValidNumber(char *p_input, int base);
int DetectBase(char *p_input);

int main() {
    char input[100];
    int base;

    printf("Enter a number: ");
    fgets(input, sizeof(input), stdin);

    base = DetectBase(input);
    
    input[strcspn(input, "\n")] = '\0'; // Remove newline character

    if (base == 8 || base == 16) {
        memmove(input, input + 2, strlen(input)); // Remove "0b" or "0x" prefix
    }

    if (IsValidNumber(input, base)) {
        printf("The number %s is valid in base %d\n", input, base);
    } else {
        printf("The number %s is not valid in base %d\n", input, base);
    }

    return 0;
}


bool IsValidNumber(char *p_input, int base) {
    static const char *base_names[] = {"Binary", "Octal", "Decimal", "Hexadecimal"};
    char *p_endPtr = p_input;
    while (*p_endPtr != '\0') {
        if (!isdigit(*p_endPtr) && toupper(*p_endPtr) < 'A' || toupper(*p_endPtr) >= 'A' + base - 10) {
            printf("The number %s is not valid in base %s because of invalid characters found here '%c'\n", p_input, base_names[base / 2 - 1], *p_endPtr);
            return false;
        }
        p_endPtr++;
    }
    return true;
}

int DetectBase(char *p_input) {
    if (strlen(p_input) >= 2 && p_input[0] == '0' && p_input[1] == 'x') {
        return 16;
    } else if (p_input[0] == '0' && p_input[1] == 'b') {
        return 8;
    } else {
        return 10;
    }
}"#
            .to_string(),
            default_stdin: "0x1A3F\n".to_string(),
        },
        CodeExample {
            identifier: "c-player-struct-io".to_string(),
            title: "Player Struct & Stdin (C)".to_string(),
            category: "Structs & I/O".to_string(),
            difficulty: "Easy".to_string(),
            source_language: SourceLanguage::C,
            description: "Typedef struct Player with puts and scanf formatted input/output.".to_string(),
            source_code: r#"#include <stdio.h>

typedef struct Player {
 int age;
 float score;
}Player;

int main(){

  Player player;

  puts("Enter P1 Age");
  scanf("%d",&player.age);

  puts("Enter P1 score");
  scanf("%f",&player.score);

  printf("Age is %d and Score is %f",player.age, player.score);
  return 0;
}"#
            .to_string(),
            default_stdin: "25\n98.5\n".to_string(),
        },
        CodeExample {
            identifier: "c-dynamic-struct-pointer".to_string(),
            title: "Dynamic Struct Pointer (C)".to_string(),
            category: "Structs & Memory".to_string(),
            difficulty: "Medium".to_string(),
            source_language: SourceLanguage::C,
            description: "Allocates a Player struct on the heap with malloc/free and reads fields via -> operator.".to_string(),
            source_code: r#"#include <stdio.h>
#include <stdlib.h>

typedef struct Player {
    int id;
    double rating;
    char rank;
} Player;

int main() {
    Player *p1 = (Player *)malloc(sizeof(Player));
    if (p1 == NULL) return 1;

    puts("Enter ID, Rating, and Rank (e.g., 101 4500.75 S):");
    scanf("%d %lf %c", &p1->id, &p1->rating, &p1->rank);

    printf("Player #%d | Rating: %.2lf | Rank: %c\n", p1->id, p1->rating, p1->rank);

    free(p1);
    return 0;
}"#
            .to_string(),
            default_stdin: "101 4500.75 S\n".to_string(),
        },
        CodeExample {
            identifier: "c-nested-structs-hero".to_string(),
            title: "Nested Structs & Hero Stats (C)".to_string(),
            category: "Structs & Memory".to_string(),
            difficulty: "Medium".to_string(),
            source_language: SourceLanguage::C,
            description: "Embeds Stats inside Character and accesses multi-level fields with formatted output.".to_string(),
            source_code: r#"#include <stdio.h>

typedef struct Stats {
    unsigned int health;
    float critChance;
} Stats;

typedef struct Character {
    char name[32];
    int level;
    Stats attributes;
} Character;

int main() {
    Character hero = {0};

    puts("Enter Hero Name (no spaces), Level, Health, and Crit Chance:");
    scanf("%31s %d %u %f", hero.name, &hero.level, &hero.attributes.health, &hero.attributes.critChance);

    printf("Hero: %s (Lvl %d) | HP: %u | Crit: %.1f%%\n",
           hero.name, hero.level, hero.attributes.health, hero.attributes.critChance);
    return 0;
}"#
            .to_string(),
            default_stdin: "Arthas 80 12500 35.5\n".to_string(),
        },
        CodeExample {
            identifier: "c-struct-array-pointer-math".to_string(),
            title: "Struct Array & Pointer Math (C)".to_string(),
            category: "Structs & Pointers".to_string(),
            difficulty: "Hard".to_string(),
            source_language: SourceLanguage::C,
            description: "Stores Match records in an array and iterates with pointer offset arithmetic.".to_string(),
            source_code: r#"#include <stdio.h>

typedef struct Match {
    long matchId;
    short kills;
    float durationMinutes;
} Match;

int main() {
    Match history[3] = {0};
    Match *ptr = history;
    int totalKills = 0;

    for (int i = 0; i < 3; i++) {
        printf("Match %d - Enter ID, Kills, Duration:\n", i + 1);
        scanf("%ld %hd %f", &(ptr + i)->matchId, &(ptr + i)->kills, &(ptr + i)->durationMinutes);
        totalKills += (ptr + i)->kills;
    }

    printf("Total Kills across 3 matches: %d (Avg: %.2f)\n", totalKills, totalKills / 3.0f);
    return 0;
}"#
            .to_string(),
            default_stdin: "1001 12 25.5\n1002 8 19.0\n1003 15 31.2\n".to_string(),
        },
        CodeExample {
            identifier: "c-tagged-union-reward".to_string(),
            title: "Tagged Union Reward (C)".to_string(),
            category: "Unions & Enums".to_string(),
            difficulty: "Hard".to_string(),
            source_language: SourceLanguage::C,
            description: "Combines a typedef enum and typedef union inside a struct to store variant reward payloads.".to_string(),
            source_code: r#"#include <stdio.h>

typedef enum RewardType { GOLD, XP_BOOST, WEAPON_CODE } RewardType;

typedef union RewardValue {
    unsigned long coins;
    double multiplier;
    char code[16];
} RewardValue;

typedef struct Reward {
    RewardType type;
    RewardValue value;
} Reward;

int main() {
    Reward drop = {0};
    int choice = 0;

    puts("Select Reward Type (0 = Gold, 1 = XP Boost, 2 = Weapon Code):");
    scanf("%d", &choice);
    drop.type = (RewardType)choice;

    if (drop.type == GOLD) {
        puts("Enter coin amount:");
        scanf("%lu", &drop.value.coins);
        printf("Awarded %lu Gold!\n", drop.value.coins);
    } else if (drop.type == XP_BOOST) {
        puts("Enter XP multiplier:");
        scanf("%lf", &drop.value.multiplier);
        printf("Awarded %.2lfx XP Boost!\n", drop.value.multiplier);
    } else {
        puts("Enter weapon code:");
        scanf("%15s", drop.value.code);
        printf("Unlocked Weapon: %s\n", drop.value.code);
    }
    return 0;
}"#
            .to_string(),
            default_stdin: "1\n2.5\n".to_string(),
        },
        CodeExample {
            identifier: "c-linked-list-leaderboard".to_string(),
            title: "Dynamic Linked List Leaderboard (C)".to_string(),
            category: "Linked Lists".to_string(),
            difficulty: "Very Complex".to_string(),
            source_language: SourceLanguage::C,
            description: "Self-referential struct linked list with dynamic allocation, traversal, and cleanup.".to_string(),
            source_code: r#"#include <stdio.h>
#include <stdlib.h>

typedef struct Node {
    int playerId;
    float score;
    struct Node *next;
} Node;

int main() {
    Node *head = NULL, *tail = NULL;
    int count = 0;

    puts("How many players to add to leaderboard?");
    scanf("%d", &count);

    for (int i = 0; i < count; i++) {
        Node *newNode = (Node *)malloc(sizeof(Node));
        if (!newNode) return 1;

        printf("Enter Player %d ID and Score:\n", i + 1);
        scanf("%d %f", &newNode->playerId, &newNode->score);
        newNode->next = NULL;

        if (!head) head = tail = newNode;
        else { tail->next = newNode; tail = newNode; }
    }

    puts("\n--- Leaderboard ---");
    Node *curr = head;
    while (curr != NULL) {
        printf("ID: %d | Score: %.2f\n", curr->playerId, curr->score);
        Node *temp = curr;
        curr = curr->next;
        free(temp);
    }
    return 0;
}"#
            .to_string(),
            default_stdin: "2\n101 95.5\n102 88.25\n".to_string(),
        },
        CodeExample {
            identifier: "c-doom-give-weapon".to_string(),
            title: "DOOM Weapon Pickup (C)".to_string(),
            category: "Game Engine".to_string(),
            difficulty: "Hard".to_string(),
            source_language: SourceLanguage::C,
            description: "Adapted from id Software's DOOM engine (d_player.h & p_inter.c) using enums and struct pointers.".to_string(),
            source_code: r#"#include <stdio.h>
#include <stdbool.h>

// Adapted from id Software's official open-source DOOM engine (d_player.h & p_inter.c)
typedef enum {
    wp_fist,
    wp_pistol,
    wp_shotgun,
    wp_chaingun,
    NUMWEAPONS
} weapontype_t;

typedef enum {
    am_clip,
    am_shell,
    NUMAMMO
} ammotype_t;

typedef struct player_s {
    int health;
    int armorpoints;
    int armortype;
    weapontype_t readyweapon;
    bool weaponowned[NUMWEAPONS];
    int ammo[NUMAMMO];
    int maxammo[NUMAMMO];
} player_t;

// Real DOOM logic for giving a player a weapon and ammo via struct pointer
bool P_GiveWeapon(player_t *player, weapontype_t weapon, bool dropped) {
    bool gaveammo = false;
    bool gaveweapon = false;

    if (weapon == wp_shotgun) {
        int amount = dropped ? 4 : 8;
        if (player->ammo[am_shell] < player->maxammo[am_shell]) {
            player->ammo[am_shell] += amount;
            if (player->ammo[am_shell] > player->maxammo[am_shell]) {
                player->ammo[am_shell] = player->maxammo[am_shell];
            }
            gaveammo = true;
        }
    }

    if (!player->weaponowned[weapon]) {
        player->weaponowned[weapon] = true;
        player->readyweapon = weapon;
        gaveweapon = true;
    }

    return (gaveweapon || gaveammo);
}

int main(void) {
    player_t p1 = {
        .health = 100,
        .armorpoints = 0,
        .armortype = 0,
        .readyweapon = wp_pistol,
        .weaponowned = { true, true, false, false },
        .ammo = { 50, 0 },
        .maxammo = { 200, 50 }
    };

    printf("Before Pickup -> Health: %d | Shells: %d | Ready Weapon ID: %d\n",
           p1.health, p1.ammo[am_shell], p1.readyweapon);

    // Passing &p1 allows P_GiveWeapon to mutate the struct using '->'
    if (P_GiveWeapon(&p1, wp_shotgun, false)) {
        puts("Picked up a shotgun!");
    }

    printf("After Pickup  -> Health: %d | Shells: %d | Ready Weapon ID: %d\n",
           p1.health, p1.ammo[am_shell], p1.readyweapon);

    return 0;
}"#
            .to_string(),
            default_stdin: String::new(),
        },
        CodeExample {
            identifier: "c-quake3-vector-physics".to_string(),
            title: "Quake III Vector Physics (C)".to_string(),
            category: "Game Engine".to_string(),
            difficulty: "Hard".to_string(),
            source_language: SourceLanguage::C,
            description: "Adapted from id Software's Quake III Arena (q_shared.h / bg_pmove.c) showing 3D vector math and gravity.".to_string(),
            source_code: r#"#include <stdio.h>

typedef float vec_t;
typedef vec_t vec3_t[3];
#define VectorCopy(a, b) ((b)[0]=(a)[0], (b)[1]=(a)[1], (b)[2]=(a)[2])
#define VectorMA(v, s, b, o) ((o)[0]=(v)[0]+(b)[0]*(s), (o)[1]=(v)[1]+(b)[1]*(s), (o)[2]=(v)[2]+(b)[2]*(s))

typedef struct {
    int commandTime;
    int pm_type;
    vec3_t origin;
    vec3_t velocity;
    int stats[16];
} playerState_t;

void PM_StepSlideMove(playerState_t *ps, float frametime) {
    // Advance player origin by velocity * delta time
    VectorMA(ps->origin, frametime, ps->velocity, ps->origin);
    // Apply simple gravity to Z axis
    ps->velocity[2] -= 800.0f * frametime;
}

int main(void) {
    playerState_t ps = {
        .origin = { 0.0f, 0.0f, 64.0f },
        .velocity = { 320.0f, 0.0f, 270.0f }
    };
    for (int frame = 1; frame <= 3; frame++) {
        PM_StepSlideMove(&ps, 0.05f);
        printf("Frame %d -> Pos: (%.1f, %.1f, %.1f) | VelZ: %.1f\n",
               frame, ps.origin[0], ps.origin[1], ps.origin[2], ps.velocity[2]);
    }
    return 0;
}"#
            .to_string(),
            default_stdin: String::new(),
        },
        CodeExample {
            identifier: "c-wolf3d-fixed-raycast".to_string(),
            title: "Wolfenstein 3D Raycasting (C)".to_string(),
            category: "Game Engine".to_string(),
            difficulty: "Hard".to_string(),
            source_language: SourceLanguage::C,
            description: "Adapted from id Software's Wolfenstein 3D (wl_draw.c) demonstrating 16.16 fixed-point ray steps.".to_string(),
            source_code: r#"#include <stdio.h>
#include <stdint.h>

typedef int32_t fixed;
#define FRACBITS 16
#define TO_FIXED(x) ((fixed)((x) * (1 << FRACBITS)))
#define TO_FLOAT(x) ((float)(x) / (1 << FRACBITS))

typedef struct {
    int tilex, tiley;
    fixed x, y;
    fixed xstep, ystep;
} raycast_t;

void StepRay(raycast_t *ray) {
    ray->x += ray->xstep;
    ray->y += ray->ystep;
    ray->tilex = ray->x >> FRACBITS;
    ray->tiley = ray->y >> FRACBITS;
}

int main(void) {
    raycast_t ray = {
        .x = TO_FIXED(2.5f),
        .y = TO_FIXED(2.5f),
        .xstep = TO_FIXED(0.75f),
        .ystep = TO_FIXED(0.50f)
    };
    for (int i = 0; i < 4; i++) {
        StepRay(&ray);
        printf("Ray Step %d -> World: (%.2f, %.2f) | Grid Tile: [%d, %d]\n",
               i + 1, TO_FLOAT(ray.x), TO_FLOAT(ray.y), ray.tilex, ray.tiley);
    }
    return 0;
}"#
            .to_string(),
            default_stdin: String::new(),
        },
        CodeExample {
            identifier: "c-doom-mobj-damage".to_string(),
            title: "DOOM Map Object Damage (C)".to_string(),
            category: "Game Engine".to_string(),
            difficulty: "Very Complex".to_string(),
            source_language: SourceLanguage::C,
            description: "Adapted from id-Software/DOOM (p_mobj.h / p_inter.c) using self-referential mobj_t structs and bitflags.".to_string(),
            source_code: r#"#include <stdio.h>

#define MF_SOLID 1
#define MF_SHOOTABLE 2
#define MF_CORPSE 4

typedef struct mobj_s {
    int type;
    int health;
    int flags;
    struct mobj_s *target; // Pointer to attacker or target
} mobj_t;

void P_DamageMobj(mobj_t *target, mobj_t *source, int damage) {
    if (!(target->flags & MF_SHOOTABLE)) return;
    target->health -= damage;
    target->target = source;
    if (target->health <= 0) {
        target->flags &= ~MF_SOLID;
        target->flags |= MF_CORPSE;
        puts("Target killed! Flags updated to MF_CORPSE.");
    }
}

int main(void) {
    mobj_t marine = { .type = 1, .health = 100, .flags = MF_SOLID | MF_SHOOTABLE, .target = NULL };
    mobj_t imp = { .type = 2, .health = 60, .flags = MF_SOLID | MF_SHOOTABLE, .target = NULL };
    printf("Imp HP before blast: %d\n", imp.health);
    P_DamageMobj(&imp, &marine, 75);
    printf("Imp HP after blast: %d | Attacker HP: %d\n", imp.health, imp.target->health);
    return 0;
}"#
            .to_string(),
            default_stdin: String::new(),
        },
        CodeExample {
            identifier: "c-quake-cvar-system".to_string(),
            title: "Quake Console Variable (Cvar) (C)".to_string(),
            category: "Game Engine".to_string(),
            difficulty: "Very Complex".to_string(),
            source_language: SourceLanguage::C,
            description: "Adapted from id Software's Quake (cvar.h / cvar.c) storing console settings with string and float values.".to_string(),
            source_code: r#"#include <stdio.h>
#include <stdlib.h>
#include <string.h>

typedef struct cvar_s {
    char *name;
    char *string;
    float value;
    struct cvar_s *next;
} cvar_t;

cvar_t *cvar_vars = NULL;

void Cvar_RegisterVariable(cvar_t *variable) {
    variable->value = (float)atof(variable->string);
    variable->next = cvar_vars;
    cvar_vars = variable;
}

void Cvar_Set(const char *var_name, const char *value) {
    for (cvar_t *var = cvar_vars; var != NULL; var = var->next) {
        if (strcmp(var->name, var_name) == 0) {
            var->string = (char *)value;
            var->value = (float)atof(value);
            return;
        }
    }
}

int main(void) {
    cvar_t sv_gravity = { "sv_gravity", "800", 0.0f, NULL };
    cvar_t fov = { "fov", "90", 0.0f, NULL };
    Cvar_RegisterVariable(&sv_gravity);
    Cvar_RegisterVariable(&fov);
    printf("Initial %s = %.1f\n", sv_gravity.name, sv_gravity.value);
    Cvar_Set("sv_gravity", "200"); // Low-gravity mode
    printf("Updated %s = %.1f\n", sv_gravity.name, sv_gravity.value);
    return 0;
}"#
            .to_string(),
            default_stdin: String::new(),
        },
        CodeExample {
            identifier: "c-sm64-object-fields".to_string(),
            title: "SM64 Object Behavior Fields (C)".to_string(),
            category: "Game Engine".to_string(),
            difficulty: "Hard".to_string(),
            source_language: SourceLanguage::C,
            description: "Adapted from n64decomp/sm64 (object_fields.h) using indexed union fields for position, velocity, and timer.".to_string(),
            source_code: r#"#include <stdio.h>
#include <stdint.h>

typedef union {
    int32_t asInt;
    float asFloat;
} ObjectField;

typedef struct Object {
    uint32_t activeFlags;
    ObjectField rawData[8]; // Indexed object attributes (pos, vel, timer)
} Object;

enum { oPosX = 0, oPosY = 1, oVelY = 2, oTimer = 3 };

void bhv_coin_bounce_loop(Object *o) {
    o->rawData[oPosY].asFloat += o->rawData[oVelY].asFloat;
    o->rawData[oVelY].asFloat -= 4.0f; // Gravity pull
    o->rawData[oTimer].asInt++;
}

int main(void) {
    Object coin = { .activeFlags = 1 };
    coin.rawData[oPosX].asFloat = 120.0f;
    coin.rawData[oPosY].asFloat = 50.0f;
    coin.rawData[oVelY].asFloat = 20.0f;
    coin.rawData[oTimer].asInt = 0;
    for (int t = 0; t < 4; t++) {
        bhv_coin_bounce_loop(&coin);
        printf("Tick %d -> Coin Y: %.1f | VelY: %.1f\n",
               coin.rawData[oTimer].asInt, coin.rawData[oPosY].asFloat, coin.rawData[oVelY].asFloat);
    }
    return 0;
}"#
            .to_string(),
            default_stdin: String::new(),
        },
        CodeExample {
            identifier: "c-float-limits-sizeof".to_string(),
            title: "Float Limits & sizeof(long double) (C)".to_string(),
            category: "Types & Compatibility".to_string(),
            difficulty: "Medium".to_string(),
            source_language: SourceLanguage::C,
            description: "Inspects long double size and <float.h> LDBL_* limits, mapped to Rust std::mem::size_of::<f64>() and f64 constants with compatibility diagnostics.".to_string(),
            source_code: r#"#include <float.h>
#include <stdio.h>

int main(void) {
    printf("sizeof(long double): %zu\n", sizeof(long double));
    printf("LDBL_MANT_DIG: %d\n", LDBL_MANT_DIG);
    printf("LDBL_DIG: %d\n", LDBL_DIG);
    printf("LDBL_MAX: %Lg\n", LDBL_MAX);
    return 0;
}"#
            .to_string(),
            default_stdin: String::new(),
        },
        CodeExample {
            identifier: "c-c99-flexible-array-member".to_string(),
            title: "Flexible Array Member (C99)".to_string(),
            category: "ISO C99/C11".to_string(),
            difficulty: "Hard".to_string(),
            source_language: SourceLanguage::C,
            description: "Pure C99 flexible array member struct with single-allocation dynamic buffer mapped to Vec<f64> in Box<Packet>.".to_string(),
            source_code: r#"#include <stdio.h>
#include <stdlib.h>

typedef struct Packet {
    int id;
    size_t length;
    double payload[]; // Pure C99 Flexible Array Member (illegal in standard C++)
} Packet;

int main(void) {
    size_t count = 3;
    Packet *pkt = malloc(sizeof(Packet) + count * sizeof(double));
    if (!pkt) return 1;

    pkt->id = 42;
    pkt->length = count;
    pkt->payload[0] = 3.14159;
    pkt->payload[1] = 2.71828;
    pkt->payload[2] = 1.61803;

    printf("Packet #%d | First payload value: %.5f\n", pkt->id, pkt->payload[0]);
    free(pkt);
    return 0;
}"#
            .to_string(),
            default_stdin: String::new(),
        },
        CodeExample {
            identifier: "c-c11-generic-selection".to_string(),
            title: "_Generic Type Selection (C11)".to_string(),
            category: "ISO C99/C11".to_string(),
            difficulty: "Hard".to_string(),
            source_language: SourceLanguage::C,
            description: "Pure C11 _Generic compile-time type dispatch macro alongside float.h limits.".to_string(),
            source_code: r#"#include <stdio.h>
#include <float.h>

#define TYPE_NAME(x) _Generic((x), \
    int: "int", \
    double: "double", \
    long double: "long double", \
    default: "unknown" \
)

int main(void) {
    int a = 10;
    double b = 20.5;
    long double c = LDBL_MAX;

    printf("a is %s | b is %s | c is %s\n", TYPE_NAME(a), TYPE_NAME(b), TYPE_NAME(c));
    return 0;
}"#
            .to_string(),
            default_stdin: String::new(),
        },
        CodeExample {
            identifier: "c-c99-compound-literal".to_string(),
            title: "Addressable Compound Literal (C99)".to_string(),
            category: "ISO C99/C11".to_string(),
            difficulty: "Medium".to_string(),
            source_language: SourceLanguage::C,
            description: "Pure C99 addressable compound literal &(Vector2){...} passed to a mutating function.".to_string(),
            source_code: r#"#include <stdio.h>

typedef struct Vector2 {
    float x;
    float y;
} Vector2;

void scale_vector(Vector2 *v, float factor) {
    v->x *= factor;
    v->y *= factor;
}

int main(void) {
    // Creates an addressable lvalue struct inline without declaring a variable name
    Vector2 *ptr = &(Vector2){ .x = 4.0f, .y = 9.0f };
    scale_vector(ptr, 2.5f);

    printf("Scaled Vector -> X: %.1f, Y: %.1f\n", ptr->x, ptr->y);
    return 0;
}"#
            .to_string(),
            default_stdin: String::new(),
        },
        CodeExample {
            identifier: "c-c99-vla-runtime-sizeof".to_string(),
            title: "2D VLA & Runtime sizeof (C99)".to_string(),
            category: "ISO C99/C11".to_string(),
            difficulty: "Hard".to_string(),
            source_language: SourceLanguage::C,
            description: "Pure C99 2D variable-length array with runtime sizeof evaluation mapped to nested Vec and runtime byte calculation.".to_string(),
            source_code: r#"#include <stdio.h>

void build_matrix(size_t rows, size_t cols) {
    long double grid[rows][cols]; // Runtime-sized 2D VLA
    grid[rows - 1][cols - 1] = 99.95L;

    printf("Total matrix bytes at runtime: %zu\n", sizeof(grid));
    printf("Bottom-right element: %.2Lf\n", grid[rows - 1][cols - 1]);
}

int main(void) {
    size_t r = 4, c = 5;
    build_matrix(r, c);
    return 0;
}"#
            .to_string(),
            default_stdin: String::new(),
        },
        CodeExample {
            identifier: "c-c99-restrict-static-array".to_string(),
            title: "restrict & [static N] Array Param (C99)".to_string(),
            category: "ISO C99/C11".to_string(),
            difficulty: "Medium".to_string(),
            source_language: SourceLanguage::C,
            description: "Pure C99 restrict pointer qualifier and [static 4] array parameter mapped to idiomatic Rust slices and &mut references.".to_string(),
            source_code: r#"#include <stdio.h>

// Guarantees 'samples' is non-NULL with at least 4 elements, and never aliases 'out'
void sum_quad(const double samples[static 4], double * restrict out) {
    *out = samples[0] + samples[1] + samples[2] + samples[3];
}

int main(void) {
    double readings[4] = { 12.5, 25.0, 37.5, 50.0 };
    double total = 0.0;

    sum_quad(readings, &total);
    printf("Quadrant sum: %.2f\n", total);
    return 0;
}"#
            .to_string(),
            default_stdin: String::new(),
        },
        CodeExample {
            identifier: "c-record-crud-array".to_string(),
            title: "Record CRUD with C Arrays (C11)".to_string(),
            category: "Standard Library CRUD".to_string(),
            difficulty: "Medium".to_string(),
            source_language: SourceLanguage::C,
            description: "Creates, reads, updates, and deletes fixed-capacity records using a C11 struct array and stdio output.".to_string(),
            source_code: r#"#include <stdio.h>

typedef struct {
    int id;
    int score;
} Record;

int main(void) {
    Record records[4] = {{1, 70}, {2, 80}};
    int count = 2;

    // Create
    records[count++] = (Record){3, 90};
    // Read
    printf("Read: id=%d score=%d\n", records[1].id, records[1].score);
    // Update
    records[1].score = 85;
    // Delete id=1 by shifting remaining records left.
    for (int i = 0; i < count - 1; i++) {
        if (records[i].id == 1) {
            for (int j = i; j < count - 1; j++) {
                records[j] = records[j + 1];
            }
            count--;
            break;
        }
    }

    printf("After CRUD (%d records):", count);
    for (int i = 0; i < count; i++) {
        printf(" %d:%d", records[i].id, records[i].score);
    }
    printf("\n");
    return 0;
}"#
            .to_string(),
            default_stdin: String::new(),
        },
        CodeExample {
            identifier: "cpp-map-crud".to_string(),
            title: "Map CRUD with STL (C++)".to_string(),
            category: "Standard Library CRUD".to_string(),
            difficulty: "Medium".to_string(),
            source_language: SourceLanguage::Cpp,
            description: "Creates, reads, updates, and deletes named scores with std::map and iterates the final collection.".to_string(),
            source_code: r#"#include <iostream>
#include <map>
#include <string>

int main() {
    std::map<std::string, int> scores;

    // Create
    scores["Ada"] = 90;
    scores["Linus"] = 80;
    // Read
    std::cout << "Read: Ada=" << scores["Ada"] << std::endl;
    // Update
    scores["Ada"] = 95;
    // Delete
    scores.erase("Linus");

    std::cout << "After CRUD:";
    for (const auto& [name, score] : scores) {
        std::cout << " " << name << "=" << score;
    }
    std::cout << std::endl;
    return 0;
}"#
            .to_string(),
            default_stdin: String::new(),
        },
    ]
}
