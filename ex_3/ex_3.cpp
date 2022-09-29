#include <assert.h>
#include <iostream>
#include <list>
#include <signal.h>
#include <sstream>
#include <vector>

using namespace std;

void not_implemented()
{
    cerr << "Not implemented." << endl;
    raise(SIGFPE);
}

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

    string buffer;
    while (getline(cin, buffer, ',')) {
        // ↑ The trailing comma is not necessary.
        input.pages.push_back(stoi(buffer));
    }

    return input;
}

#define IDLE -1
/// 页表，数字表示物理页框号，`IDLE`表示空闲
using PageTable = vector<int>;
using Page = PageTable::iterator;

struct PageChange {
    PageTable table;
    bool hit;

    PageChange(PageTable table, bool hit) : table(table), hit(hit) {}
};

void write_outputs(vector<PageChange> changes)
{
    unsigned int n_page_faults = 0;

    bool is_first_change = true;
    for (auto &&c : changes) {
        // 1. separator
        if (is_first_change) {
            is_first_change = false;
        } else {
            cout << "/";
        }

        // 2. page table
        for (auto &&i : c.table) {
            if (i == IDLE) {
                cout << "-";
            } else {
                cout << i;
            }

            cout << ",";
        }

        // 3. hit or miss
        cout << c.hit ? "1" : "0";

        // 4. count page faults
        n_page_faults += !c.hit;
    }

    cout << "\n"
         << n_page_faults << endl;
}

class Manager
{
protected:
    PageTable table;

public:
    Manager(unsigned int n_frames) : table(PageTable(n_frames, IDLE)) {}

    vector<PageChange> request(const vector<int> &requests)
    {
        vector<PageChange> changes;

        const auto request_end = requests.end();
        for (auto r = requests.begin(); r != request_end; ++r) {
            const bool hit = this->can_hit(*r);

            if (!hit) {
                // Find where to insert / swap
                auto where = this->find_idle();
                if (where == this->table.end()) {
                    where = this->next_to_swap(r);
                }

                // insert / swap
                this->swap(where, *r);
            }

            changes.push_back(PageChange(this->table, hit));
        }

        return changes;
    }

protected:
    virtual void swap(Page where, int frame)
    {
        *where = frame;
    }

    /**
     * @brief Find an idle page in the page table
     *
     * @return PageTable::iterator `end` if none
     */
    Page find_idle()
    {
        const auto end = this->table.end();

        auto p = this->table.begin();
        while (p != end && *p != IDLE) {
            ++p;
        }
        return p;
    }

    virtual Page next_to_swap(const vector<int>::const_iterator &current_request) = 0;

    bool can_hit(int request)
    {
        for (auto &&p : this->table) {
            if (p == request) {
                return true;
            }
        }
        return false;
    }
};

class ManagerFIFO : public Manager
{
protected:
    list<Page> history;

public:
    ManagerFIFO(unsigned int n_frames) : Manager(n_frames) {}

protected:
    Page next_to_swap(const vector<int>::const_iterator &current_request)
    {
        return this->history.front();
    }

    void swap(Page where, int frame)
    {
        if (*where != IDLE) {
            this->history.pop_front();
        }

        Manager::swap(where, frame);
        this->history.push_back(where);
    }
};

int main()
{
    auto input = read_inputs();

    Manager *manager = nullptr;
    switch (input.policy) {
    case Policy::FirstInFirstOut:
        manager = new ManagerFIFO(input.n_frames);
        break;

    default:
        not_implemented();
        break;
    }

    write_outputs(manager->request(input.pages));
    delete manager;

    return 0;
}
