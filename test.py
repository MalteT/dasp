#!/usr/bin/env python3

from clingo.application import clingo_main, Application
from clingo.control import Control
from clingo.symbol import Function
from typing import Sequence
import sys


def add_argument_program(arg: str) -> str:
    return f"""
        #external arg({arg}).

        %% Guess a set S \\subseteq A
        in({arg}) :- not out({arg}), arg({arg}).
        out({arg}) :- not in({arg}), arg({arg}).

        #external defeated({arg}) : arg({arg}).

        #show {arg}: in({arg}).
    """


def add_attack_program(fro: str, to: str) -> str:
    """Must not be called with undefined arguments, add them first!"""
    return f"""
        #external att({fro}, {to}).

        %% S has to be conflict-free
        :- in({fro}), in({to}), att({fro}, {to}).

        %% The argument x is defeated by the set S
        defeated({to}) :- in({fro}), att({fro}, {to}).

        %% The argument x is not defended by S
        not_defended({to}) :- att({fro}, {to}), not defeated({fro}).
        :- in({to}), not_defended({to}).
    """


class App(Application):
    def __init__(self):
        self.update_count = 0

    def add_update(self, ctl: Control, content: str):
        update_name = f"update_{self.update_count}"
        ctl.add(update_name, [], content)
        ctl.ground([(update_name, [])])
        self.update_count += 1

    def add_argument(self, ctl: Control, arg: str):
        content = add_argument_program(arg)
        print('===', content, '===', sep='\n')
        self.add_update(ctl, content)
        ctl.assign_external(Function("arg", [Function(arg, [])]), True)

    def add_attack(self, ctl: Control, fro: str, to: str):
        content = add_attack_program(fro, to)
        print('===', content, '===', sep='\n')
        self.add_update(ctl, content)
        ctl.assign_external(
            Function("att", [Function(fro, []), Function(to, [])]), True)

    def main(self, ctl: Control, files: Sequence[str]):
        ctl.add("show", [], """
            #defined not_defended/1.
            #defined defeated/1.
            #defined in/1.
            #show.
            #show X : in(X).
        """)
        ctl.ground([("show", [])])
        print("\nExpect: ()")
        ctl.solve()

        self.add_argument(ctl, "a")
        self.add_argument(ctl, "b")
        self.add_attack(ctl, "a", "b")
        print("\nExpect: (), (a)")
        ctl.solve()

        self.add_argument(ctl, "c")
        print("\nExpect: (), (a), (c), (a,c)")
        ctl.solve()

        self.add_attack(ctl, "c", "a")
        print("\nExpect: (), (c), (b,c)")
        ctl.solve()

        # ctl.add("update_2", [], add_attack("c", "a"))
        # print("Expect: (), (c), (b,c)")
        # ctl.ground([("update_2", [])])
        # ctl.solve()

        # ctl.add("update_3", [], add_attack("b", "c"))
        # print("Expect: ()")
        # ctl.ground([("update_3", [])])
        # ctl.solve()


clingo_main(App(), sys.argv[:2])
