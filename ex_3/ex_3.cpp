#include <assert.h>
#include <iostream>
#include <vector>
using namespace std;

enum Policy {
    Optimal = 1,
    FirstInFirstOut = 2,
    LeastRecentlyUsed = 3,
};

struct Input {
    Policy policy;
    unsigned int n_frames;
    vector<int> pages;
};

Input read_inputs()
{
    Input input;

    int policy;
    cin >> policy;
    assert(1 <= policy && policy <= 3);
    input.policy = Policy(policy);

    cin >> input.n_frames;

    int page_index;
    while (EOF != scanf("%d,", &page_index)) {
        // ↑ The trailing comma is not necessary.
        input.pages.push_back(page_index);
    }

    return input;
}

int main()
{
    auto input = read_inputs();
    for (auto &&p : input.pages) {
        cout << p;
    }

    return 0;
}
