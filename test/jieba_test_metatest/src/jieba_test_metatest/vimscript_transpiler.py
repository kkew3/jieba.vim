# Copyright 2026 Kaiwen Wu. All Rights Reserved.
#
# Licensed under the Apache License, Version 2.0 (the "License"); you may not
# use this file except in compliance with the License. You may obtain a copy of
# the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS, WITHOUT
# WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied. See the
# License for the specific language governing permissions and limitations under
# the License.

from typing import Any


class VimExpr:
    def __init__(self, ty: str, arg1, arg2=None):
        self.ty = ty
        self.arg1 = arg1
        self.arg2 = arg2

    @classmethod
    def int_(cls, v: int) -> "VimExpr":
        if not isinstance(v, int):
            raise TypeError(f"invalid type: {type(v)}")
        return cls("int", v)

    @classmethod
    def literal(cls, v: str) -> "VimExpr":
        if not isinstance(v, str):
            raise TypeError(f"invalid type: {type(v)}")
        v = (
            v.replace("\n", "\\<Newline>")
            .replace("\r", "\\<CR>")
            .replace("\t", "\\<Tab>")
        )
        return cls("literal", v)

    @classmethod
    def var(cls, v: str) -> "VimExpr":
        if not isinstance(v, str):
            raise TypeError(f"invalid type: {type(v)}")
        return cls("var", v)

    @classmethod
    def list_(cls, lst: list) -> "VimExpr":
        return cls("list", [cls.into_vimexpr(a) for a in lst])

    @classmethod
    def dict_(cls, dct: dict[str, Any]) -> "VimExpr":
        return cls(
            "dict",
            {cls.literal(k): cls.into_vimexpr(v) for k, v in dct.items()},
        )

    @classmethod
    def cmd(cls, name: str, *args) -> "VimExpr":
        return cls("cmd", cls.var(name), [cls.into_vimexpr(a) for a in args])

    @classmethod
    def into_vimexpr(cls, v) -> "VimExpr":
        if isinstance(v, VimExpr):
            return v
        if isinstance(v, VimExprBuilder):
            return v.vim_expr
        if isinstance(v, int):
            return cls.int_(v)
        if isinstance(v, str):
            return cls.literal(v)
        if isinstance(v, list):
            return cls.list_(v)
        if isinstance(v, dict):
            return cls.dict_(v)
        raise TypeError(f"unsupported type: {type(v)}")

    def __str__(self):
        if self.ty == "int":
            return f"{self.arg1}"
        if self.ty == "literal":
            escaped = f"{self.arg1}".replace('"', '\\"')
            return f'"{escaped}"'
        if self.ty == "var":
            return f"{self.arg1}"
        if self.ty == "list":
            return "[" + ", ".join(f"{x}" for x in self.arg1) + "]"
        if self.ty == "dict":
            return (
                "{" + ", ".join(f"{k}: {v}" for k, v in self.arg1.items()) + "}"
            )
        if self.ty == "cmd":
            return f"{self.arg1}" + "".join(f" {a}" for a in self.arg2)
        if self.ty in ("dict_access", "list_index"):
            return f"{self.arg1}[{self.arg2}]"
        if self.ty == "func_call":
            args = ", ".join(f"{x}" for x in self.arg2)
            return f"{self.arg1}({args})"
        if self.ty == "group":
            return f"({self.arg1})"
        if self.ty == "not":
            return f"!({self.arg1})"

        try:
            bin_op = {
                "concat": ".",
                "eq": "==#",
                "ne": "!=#",
                "gt": ">",
                "ge": ">=",
                "lt": "<",
                "le": "<=",
                "and": "&&",
                "or": "||",
            }[self.ty]
        except KeyError as err:
            raise ValueError(f"invalid VimExpr type: {self.ty}") from err
        return f"{self.arg1} {bin_op} {self.arg2}"

    def __eq__(self, other):
        if isinstance(other, VimExpr):
            return (
                self.ty == other.ty
                and self.arg1 == other.arg1
                and self.arg2 == other.arg2
            )
        return NotImplemented

    def __hash__(self):
        return hash((self.ty, self.arg1, self.arg2))

    def __repr__(self):
        if self.arg2 is None:
            return f"VimExpr::{self.ty}({self.arg1!r})"
        return f"VimExpr::{self.ty}({self.arg1!r}, {self.arg2!r})"

    def build(self) -> "VimExprBuilder":
        return VimExprBuilder(self, _raw=True)


# Shortcut constructors:


def int_(obj: int) -> "VimExprBuilder":
    return VimExpr.int_(obj).build()


def lit(obj: str) -> "VimExprBuilder":
    return VimExpr.literal(obj).build()


def var(obj: str) -> "VimExprBuilder":
    return VimExpr.var(obj).build()


# Enf of shortcut constructors.


class VimExprBuilder:
    def __init__(self, vim_expr, *, _raw=False):
        if _raw:
            self.vim_expr = vim_expr
        else:
            self.vim_expr = to_vim_expr(vim_expr)

    def __add__(self, other) -> "VimExprBuilder":
        if isinstance(other, VimExpr):
            return VimExprBuilder(VimExpr("concat", self.vim_expr, other))
        return self + to_vim_expr(other)

    def __eq__(self, other) -> "VimExprBuilder":
        if isinstance(other, VimExpr):
            return VimExprBuilder(VimExpr("eq", self.vim_expr, other))
        return self == to_vim_expr(other)

    def __ne__(self, other) -> "VimExprBuilder":
        if isinstance(other, VimExpr):
            return VimExprBuilder(VimExpr("ne", self.vim_expr, other))
        return self != to_vim_expr(other)

    def __gt__(self, other) -> "VimExprBuilder":
        if isinstance(other, VimExpr):
            return VimExprBuilder(VimExpr("gt", self.vim_expr, other))
        return self > to_vim_expr(other)

    def __ge__(self, other) -> "VimExprBuilder":
        if isinstance(other, VimExpr):
            return VimExprBuilder(VimExpr("ge", self.vim_expr, other))
        return self >= to_vim_expr(other)

    def __lt__(self, other) -> "VimExprBuilder":
        if isinstance(other, VimExpr):
            return VimExprBuilder(VimExpr("lt", self.vim_expr, other))
        return self < to_vim_expr(other)

    def __le__(self, other) -> "VimExprBuilder":
        if isinstance(other, VimExpr):
            return VimExprBuilder(VimExpr("le", self.vim_expr, other))
        return self <= to_vim_expr(other)

    def __and__(self, other) -> "VimExprBuilder":
        if isinstance(other, VimExpr):
            return VimExprBuilder(VimExpr("and", self.vim_expr, other))
        return self & to_vim_expr(other)

    def __or__(self, other) -> "VimExprBuilder":
        if isinstance(other, VimExpr):
            return VimExprBuilder(VimExpr("or", self.vim_expr, other))
        return self | to_vim_expr(other)

    def __invert__(self) -> "VimExprBuilder":
        return VimExprBuilder(VimExpr("not", self.vim_expr))

    def __getitem__(self, other) -> "VimExprBuilder":
        if isinstance(other, VimExpr):
            if self.vim_expr.ty in ("var", "dict") and other.ty == "literal":
                return VimExprBuilder(
                    VimExpr("dict_access", self.vim_expr, other)
                )
            if self.vim_expr.ty in ("var", "list") and other.ty == "int":
                return VimExprBuilder(
                    VimExpr("list_index", self.vim_expr, other)
                )
            raise TypeError(
                f"invalid (self, index) types: "
                f"(VimExpr::{self.vim_expr.ty}, VimExpr::{other.ty})"
            )
        return self[to_vim_expr(other)]

    def __call__(self, *args) -> "VimExprBuilder":
        if self.vim_expr.ty != "var":
            raise TypeError(f"invalid self type: VimExpr::{self.vim_expr.ty}")
        return VimExprBuilder(
            VimExpr("func_call", self.vim_expr, [to_vim_expr(x) for x in args])
        )

    def group(self) -> "VimExprBuilder":
        return VimExprBuilder(VimExpr("group", self.vim_expr))

    def __str__(self):
        return str(self.vim_expr)

    def unwrap(self):
        return self.vim_expr


def to_vim_expr(obj) -> VimExpr:
    return VimExpr.into_vimexpr(obj)


class LuaExpr:
    def __init__(self, ty: str, arg1, arg2=None):
        self.ty = ty
        self.arg1 = arg1
        self.arg2 = arg2

    @classmethod
    def int_(cls, v: int):
        if not isinstance(v, int):
            raise TypeError(f"invalid type: {type(v)}")
        return cls("int", v)

    @classmethod
    def literal(cls, v: str):
        if not isinstance(v, str):
            raise TypeError(f"invalid type: {type(v)}")
        if "\\" in v:
            raise ValueError(f"unsupported char '\\' in LuaExpr::literal: {v}")
        return cls("literal", v)

    @classmethod
    def vim_var(cls, v: str):
        if not isinstance(v, str):
            raise TypeError(f"invalid type: {type(v)}")
        return cls("vim_var", v)

    @classmethod
    def lua_var(cls, v: str):
        if isinstance(v, VimExpr) and v.ty == "literal":
            return cls("lua_var", v.arg1)
        if not isinstance(v, str):
            raise TypeError(f"invalid type: {type(v)}")
        return cls("lua_var", v)

    @classmethod
    def list_(cls, lst: list):
        return cls("list", [cls.into_luaexpr(a) for a in lst])

    @classmethod
    def dict_(cls, dct: dict[str, Any]):
        return cls(
            "dict",
            {cls.lua_var(k): cls.into_luaexpr(v) for k, v in dct.items()},
        )

    @classmethod
    def wrap_vim(cls, v) -> "LuaExpr":
        v = to_vim_expr(v)
        if v.ty == "int":
            return cls.int_(v.arg1)
        if v.ty == "literal":
            return cls.literal(v.arg1)
        if v.ty == "var":
            return cls.vim_var(v.arg1)
        if v.ty == "list":
            return cls.list_(v.arg1)
        if v.ty == "dict":
            return cls.dict_(v.arg1)
        if v.ty == "cmd":
            raise ValueError("cannot convert VimExpr::cmd to lua")
        if v.ty == "dict_access":
            return cls(
                "dict_access",
                cls.into_luaexpr(v.arg1),
                cls.into_luaexpr(v.arg2),
            )
        if v.ty == "list_index":
            raise ValueError("cannot convert VimExpr::list_index to lua")
        if v.ty == "func_call":
            return cls(
                "vim_func_call",
                cls.into_luaexpr(v.arg1),
                cls.list_(v.arg2).arg1,
            )
        if v.ty == "group":
            return cls("group", cls.into_luaexpr(v.arg1))
        if v.ty == "not":
            return cls("not", cls.into_luaexpr(v.arg1))
        if v.ty in {"concat", "eq", "ne", "gt", "ge", "lt", "le", "and", "or"}:
            return cls(v.ty, cls.into_luaexpr(v.arg1), cls.into_luaexpr(v.arg2))
        raise ValueError(f"invalid VimExpr type: {v.ty}")

    @classmethod
    def into_luaexpr(cls, v):
        if isinstance(v, LuaExpr):
            return v
        if isinstance(v, LuaExprBuilder):
            return v.lua_expr
        if isinstance(v, int):
            return cls.int_(v)
        if isinstance(v, str):
            return cls.literal(v)
        if isinstance(v, list):
            return cls.list_(v)
        if isinstance(v, dict):
            return cls.dict_(v)
        if isinstance(v, VimExpr):
            return cls.wrap_vim(v)
        if isinstance(v, VimExprBuilder):
            return cls.wrap_vim(v.vim_expr)
        raise TypeError(f"unsupported type: {type(v)}")

    def __str__(self):
        if self.ty == "int":
            return f"{self.arg1}"
        if self.ty == "literal":
            escaped = (
                f"{self.arg1}".replace('"', '\\"')
                .replace("\n", "\\n")
                .replace("\t", "\\t")
                .replace("\r", "\\r")
            )
            return f'"{escaped}"'
        if self.ty == "vim_var":
            if self.arg1[:1] == "&":
                return f"vim.o.{self.arg1[1:]}"
            if self.arg1[:2] not in ("g:", "w:", "b:", "t:", "v:"):
                raise ValueError(
                    f"cannot convert VimExpr::var({self.arg1}) to lua"
                )
            if not self.arg1[2:]:
                raise ValueError(
                    f"cannot convert VimExpr::var({self.arg1}) to lua"
                )
            scope = self.arg1[0]
            name = self.arg1[2:]
            return f"vim.{scope}.{name}"
        if self.ty == "lua_var":
            return f"{self.arg1}"
        if self.ty == "list":
            return "{" + ", ".join(f"{x}" for x in self.arg1) + "}"
        if self.ty == "dict":
            return (
                "{"
                + ", ".join(f"{k} = {v}" for k, v in self.arg1.items())
                + "}"
            )
        if self.ty == "dict_access":
            return f"{self.arg1}[{self.arg2}]"
        if self.ty == "vim_func_call":
            assert isinstance(self.arg1, LuaExpr) and self.arg1.ty == "vim_var"
            args = ", ".join(f"{x}" for x in self.arg2)
            return f"vim.fn.{self.arg1.arg1}({args})"
        if self.ty == "lua_func_call":
            assert isinstance(self.arg1, LuaExpr) and self.arg1.ty == "lua_var"
            args = ", ".join(f"{x}" for x in self.arg2)
            return f"{self.arg1.arg1}({args})"
        if self.ty == "group":
            return f"({self.arg1})"
        if self.ty == "not":
            return f"not ({self.arg1})"

        try:
            bin_op = {
                "concat": "..",
                "eq": "==",
                "ne": "~=",
                "gt": ">",
                "ge": ">=",
                "lt": "<",
                "le": "<=",
                "and": "and",
                "or": "or",
            }[self.ty]
        except KeyError as err:
            raise ValueError(f"invalid LuaExpr type: {self.ty}") from err
        return f"{self.arg1} {bin_op} {self.arg2}"

    def __eq__(self, other):
        if isinstance(other, LuaExpr):
            return (
                self.ty == other.ty
                and self.arg1 == other.arg1
                and self.arg2 == other.arg2
            )
        return NotImplemented

    def __hash__(self):
        return hash((self.ty, self.arg1, self.arg2))

    def __repr__(self):
        if self.arg2 is None:
            return f"LuaExpr::{self.ty}({self.arg1!r})"
        return f"LuaExpr::{self.ty}({self.arg1!r}, {self.arg2!r})"

    def build(self) -> "LuaExprBuilder":
        return LuaExprBuilder(self, _raw=True)


def to_lua_expr(obj) -> LuaExpr:
    return LuaExpr.into_luaexpr(obj)


class LuaExprBuilder:
    def __init__(self, lua_expr, *, _raw=False):
        if _raw:
            self.lua_expr = lua_expr
        else:
            self.lua_expr = to_lua_expr(lua_expr)

    def __add__(self, other) -> "LuaExprBuilder":
        if isinstance(other, LuaExpr):
            return LuaExprBuilder(LuaExpr("concat", self.lua_expr, other))
        return self + to_lua_expr(other)

    def __eq__(self, other) -> "LuaExprBuilder":
        if isinstance(other, LuaExpr):
            return LuaExprBuilder(LuaExpr("eq", self.lua_expr, other))
        return self == to_lua_expr(other)

    def __ne__(self, other) -> "LuaExprBuilder":
        if isinstance(other, LuaExpr):
            return LuaExprBuilder(LuaExpr("ne", self.lua_expr, other))
        return self != to_lua_expr(other)

    def __gt__(self, other) -> "LuaExprBuilder":
        if isinstance(other, LuaExpr):
            return LuaExprBuilder(LuaExpr("gt", self.lua_expr, other))
        return self > to_lua_expr(other)

    def __ge__(self, other) -> "LuaExprBuilder":
        if isinstance(other, LuaExpr):
            return LuaExprBuilder(LuaExpr("ge", self.lua_expr, other))
        return self >= to_lua_expr(other)

    def __lt__(self, other) -> "LuaExprBuilder":
        if isinstance(other, LuaExpr):
            return LuaExprBuilder(LuaExpr("lt", self.lua_expr, other))
        return self < to_lua_expr(other)

    def __le__(self, other) -> "LuaExprBuilder":
        if isinstance(other, LuaExpr):
            return LuaExprBuilder(LuaExpr("le", self.lua_expr, other))
        return self <= to_lua_expr(other)

    def __and__(self, other) -> "LuaExprBuilder":
        if isinstance(other, LuaExpr):
            return LuaExprBuilder(LuaExpr("and", self.lua_expr, other))
        return self & to_lua_expr(other)

    def __or__(self, other) -> "LuaExprBuilder":
        if isinstance(other, LuaExpr):
            return LuaExprBuilder(LuaExpr("or", self.lua_expr, other))
        return self | to_lua_expr(other)

    def __invert__(self) -> "LuaExprBuilder":
        return LuaExprBuilder(LuaExpr("not", self.lua_expr))

    def __getitem__(self, other) -> "LuaExprBuilder":
        if isinstance(other, LuaExpr):
            if (
                self.lua_expr.ty in ("vim_var", "lua_var", "dict")
                and other.ty == "literal"
            ):
                return LuaExprBuilder(
                    LuaExpr("dict_access", self.lua_expr, other)
                )
            # Intentionally don't support list_access because lua list index
            # is inconsistent with vim.
            raise TypeError(
                f"invalid (self, index) types: "
                f"(LuaExpr::{self.lua_expr.ty}, LuaExpr::{other.ty})"
            )
        return self[to_lua_expr(other)]

    def __call__(self, *args) -> "LuaExprBuilder":
        if self.lua_expr.ty not in ("vim_var", "lua_var"):
            raise TypeError(f"invalid self type: LuaExpr::{self.lua_expr.ty}")
        if self.lua_expr.ty == "vim_var":
            return LuaExprBuilder(
                LuaExpr(
                    "vim_func_call",
                    self.lua_expr,
                    [to_lua_expr(x) for x in args],
                )
            )
        return LuaExprBuilder(
            LuaExpr(
                "lua_func_call", self.lua_expr, [to_lua_expr(x) for x in args]
            )
        )

    def group(self) -> "LuaExprBuilder":
        return LuaExprBuilder(LuaExpr("group", self.lua_expr))

    def __str__(self):
        return str(self.lua_expr)

    def unwrap(self):
        return self.lua_expr


def json_encoded(obj):
    if isinstance(obj, VimExpr):
        escape = VimExpr.var("escape").build()
        json_encode = VimExpr.var("json_encode").build()
        return escape(json_encode(obj), "\\\\")
    if isinstance(obj, LuaExpr):
        json_encode = LuaExpr.vim_var("json_encode").build()
        return json_encode(obj)
    if isinstance(obj, (VimExprBuilder, LuaExprBuilder)):
        return json_encoded(obj.unwrap())
    raise TypeError(f"invalid type: {type(obj)}")


def echo(err: bool, obj):
    if isinstance(obj, VimExpr):
        shellescape = VimExpr.var("shellescape").build()
        exe_str = VimExpr.literal("!echo ").build() + shellescape(obj, 1)
        if err:
            exe_str = exe_str + " >&2"
        return VimExpr.cmd("execute", exe_str)
    if isinstance(obj, LuaExpr):
        if err:
            write = LuaExpr.lua_var("io.stderr:write").build()
        else:
            write = LuaExpr.lua_var("io.write").build()
        return write(obj.build() + "\n")
    if isinstance(obj, (VimExprBuilder, LuaExprBuilder)):
        return echo(err, obj.unwrap())
    raise TypeError(f"invalid type: {type(obj)}")


def not_eq_test_as_str(msg: str, actual, expected):
    actual_vim = to_vim_expr(actual)
    expected_vim = to_vim_expr(expected)
    actual_lua = LuaExpr.wrap_vim(actual_vim)
    expected_lua = LuaExpr.wrap_vim(expected_vim)
    msg_vim = VimExpr.literal(msg).build()
    msg_lua = LuaExpr.wrap_vim(msg_vim).build()

    content_vim = (
        msg_vim
        + " actual:: "
        + json_encoded(actual_vim)
        + " expected:: "
        + expected_vim
    )
    content_lua = (
        msg_lua
        + " actual:: "
        + json_encoded(actual_lua)
        + " expected:: "
        + expected_lua
    )
    echo_vim = echo(True, content_vim)
    echo_lua = echo(True, content_lua)

    return f"""\
if {actual_vim} !=# {expected_vim}
    if has("nvim")
        lua <<EOF
{echo_lua}
EOF
    else
        {echo_vim}
    endif
endif
"""
