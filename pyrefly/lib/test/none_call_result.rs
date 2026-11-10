/*
 * Copyright (c) Meta Platforms, Inc. and affiliates.
 *
 * This source code is licensed under the MIT license found in the
 * LICENSE file in the root directory of this source tree.
 */

use crate::test::util::TestEnv;
use crate::testcase;

fn env() -> TestEnv {
    TestEnv::new().enable_none_call_result_error()
}

testcase!(
    call_targets,
    env(),
    r#"
import functools
from typing import Callable, Protocol, overload

def notify() -> None: ...

class Base:
    def run(self) -> None: ...
    @classmethod
    def make(cls) -> None: ...
    @staticmethod
    def helper() -> None: ...

class Child(Base):
    def run(self) -> None:
        a = super().run()  # E: declared as `None`

class Hook(Protocol):
    def __call__(self) -> None: ...

class Plain:
    def __call__(self) -> None: ...

@overload
def log(value: int) -> None: ...
@overload
def log(value: str) -> None: ...
def log(value: int | str) -> None: ...

def use(base: Base, hook: Hook, plain: Plain, callback: Callable[[], None]) -> None:
    b = notify()  # E: declared as `None`
    c = base.run()  # E: declared as `None`
    e = hook()  # E: declared as `None`
    f = callback()  # E: declared as `None`
    g = functools.partial(notify)()  # E: declared as `None`
    h = log(1)  # E: declared as `None`
    i = Base.run(base)  # E: declared as `None`
    j = Base.make()  # E: declared as `None`
    k = Base.helper()  # E: declared as `None`
    l = plain()  # E: declared as `None`
    m = [1].append(2)  # E: declared as `None`
    alias = notify
    n = alias()  # E: declared as `None`
"#,
);

testcase!(
    value_positions,
    env(),
    r#"
def flag() -> bool: ...
def notify() -> None: ...

value = notify()  # E: declared as `None`
_ = notify()  # E: declared as `None`
print(notify())  # E: declared as `None`
values = [notify()]  # E: declared as `None`
flag() and notify()  # E: declared as `None`
choice = 1 if flag() else notify()  # E: declared as `None`
1 if notify() else 2  # E: declared as `None`
text = f"{notify()}"  # E: declared as `None`
annotated: object = notify()  # E: declared as `None`
pair = (None, notify())  # E: declared as `None`
equal = notify() == 1  # E: declared as `None`
negated = not notify()  # E: declared as `None`
either = flag() or notify()  # E: declared as `None`
attribute = notify().foo  # E: declared as `None`  # E: Object of class `NoneType` has no attribute `foo`
"#,
);

testcase!(
    statement_positions,
    env(),
    r#"
from typing import Iterator

def notify() -> None: ...

if notify():  # E: declared as `None`
    pass  # E: This code is unreachable
while notify():  # E: declared as `None`
    break
for item in notify():  # E: declared as `None`  # E: Type `None` is not iterable
    pass

def returns_object() -> object:
    return notify()  # E: declared as `None`

def yields() -> Iterator[None]:
    yield notify()  # E: declared as `None`

def raises() -> None:
    raise notify()  # E: declared as `None`  # E: Expression `notify()` has type `None`, expected `BaseException`
"#,
);

testcase!(
    await_expression,
    env(),
    r#"
async def notify_later() -> None: ...
def notify() -> None: ...

async def main() -> None:
    value = await notify_later()  # E: declared as `None`
    await notify_later()
    coroutine = notify_later()
    other = await notify()  # E: Type `None` is not awaitable
    return await notify_later()
"#,
);

testcase!(
    union_callees,
    env(),
    r#"
def first() -> None: ...
def second() -> None: ...
async def first_later() -> None: ...
async def second_later() -> None: ...
async def unannotated_later():
    pass

async def main(flag: bool) -> None:
    pick = first if flag else second
    a = pick()  # E: declared as `None`
    pick_later = first_later if flag else second_later
    b = await pick_later()  # E: declared as `None`
    c = await unannotated_later()
"#,
);

testcase!(
    bug = "Awaiting a callable typed as returning `Awaitable[None]` should be reported",
    awaited_callable,
    env(),
    r#"
from typing import Any, Awaitable, Callable, Coroutine

async def main(
    awaitable: Callable[[], Awaitable[None]],
    coroutine: Callable[[], Coroutine[Any, Any, None]],
) -> None:
    a = await awaitable()
    b = await coroutine()
"#,
);

testcase!(
    function_bodies,
    env(),
    r#"
def notify() -> None: ...

def compute() -> int:
    return notify()  # E: declared as `None`  # E: Returned type `None` is not assignable

def forward():
    out = notify()  # E: declared as `None`
    return out
"#,
);

testcase!(
    discarding_sites,
    env(),
    r#"
from typing import Any, Callable

def flag() -> bool: ...
def notify() -> None: ...
def other() -> None: ...

notify()
flag() if flag() else notify()
notify() if flag() else None
callback: Callable[[], None] = lambda: notify()
choose: Callable[[], None] = lambda: notify() if flag() else other()

def returns_none() -> None:
    return notify() if flag() else other()

def returns_any() -> Any:
    return notify()

def unannotated():
    return notify()

def unreachable() -> int:
    raise ValueError()
    return notify()  # E: This code is unreachable

class Base:
    def save(self) -> None: ...

class Child(Base):
    def save(self):
        return super().save()
"#,
);

testcase!(
    special_calls,
    env(),
    r#"
from typing import assert_type, cast, reveal_type

def notify() -> None: ...

assert_type(notify(), None)
reveal_type(notify())  # E: revealed type: None
value = cast(object, notify())
"#,
);

testcase!(
    generator_return,
    env(),
    r#"
from typing import Generator, Iterable, Iterator

def notify() -> None: ...

def gen_none() -> Generator[int, None, None]:
    yield 1
    return notify()

def gen_iterator() -> Iterator[int]:
    yield 1
    return notify()

def gen_iterable() -> Iterable[int]:
    yield 1
    return notify()

def gen_unannotated():
    yield 1
    return notify()

def gen_int() -> Generator[int, None, int]:
    yield 1
    return notify()  # E: declared as `None`  # E: Returned type `None` is not assignable
"#,
);

// `Runner.__call__` infers its `None` from the body, so not every member of the union declares
// it, even though the call produces `None`.
testcase!(
    not_declared_none,
    env(),
    r#"
from typing import Callable, NoReturn, TypeVar

T = TypeVar("T")

def maybe() -> int | None: ...
def unannotated():
    pass
def stop() -> NoReturn: ...
def identity(value: T) -> T: ...

class Service:
    def __init__(self) -> None: ...

class Runner:
    def __call__(self):
        pass

def use(
    mixed: Callable[[], None] | Callable[[], int],
    partly_inferred: Callable[[], None] | Runner,
) -> None:
    a = maybe()
    b = unannotated()
    c = stop()
    d = identity(None)
    e = Service()
    f = mixed()
    g = partly_inferred()
"#,
);

testcase!(
    off_by_default,
    TestEnv::new(),
    r#"
def notify() -> None: ...

value = notify()
"#,
);

testcase!(
    bug = "A `None` obtained by substituting a type parameter is reported as declared",
    substituted_none,
    env(),
    r#"
from typing import Generic, TypeVar

T = TypeVar("T")

class Box(Generic[T]):
    def get(self) -> T: ...

value = Box[None]().get()  # E: declared as `None`
callback = lambda: print()
result = callback()  # E: declared as `None`
"#,
);

testcase!(
    bug = "Selecting the `-> None` overload of a mixed overload should be reported",
    mixed_overload,
    env(),
    r#"
from typing import overload

@overload
def pick(value: int) -> None: ...
@overload
def pick(value: str) -> str: ...
def pick(value: int | str) -> str | None: ...

first = pick(1)
"#,
);

testcase!(
    bug = "A `@singledispatch` dispatcher call should be checked",
    singledispatch_dispatcher,
    env(),
    r#"
import functools

@functools.singledispatch
def handle(value: object) -> None: ...

result = handle(1)
"#,
);

// The decorated function keeps the metadata of the undecorated `def`, whose return type is
// inferred rather than written.
testcase!(
    bug = "A ParamSpec-decorated `def` without a return annotation should be reported",
    paramspec_decorated_unannotated_def,
    env(),
    r#"
from typing import Callable, ParamSpec, TypeVar

P = ParamSpec("P")
R = TypeVar("R")

def logged(function: Callable[P, R]) -> Callable[P, R]: ...

@logged
def ping():
    pass

value = ping()
"#,
);
