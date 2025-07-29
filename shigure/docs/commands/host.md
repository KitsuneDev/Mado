# host Commands

| Command | Args | Return | Description |
|---------|------|--------|-------------|
| [execute_bang](#execute_bang) | `String` | `()` | **Rainmeter Only** Execute a Rainmeter Bang Example: [!SetVariable SomeVar 10] |
| [get_host](#get_host) | `()` | `String` |  |
| [get_skin_name](#get_skin_name) | `()` | `String` | **Rainmeter Only** |
| [get_variable](#get_variable) | `String` | `String` | **Rainmeter Only** Replace a Rainmeter Variable by its value var - The Var String, like: #MyVar# |
| [read_double](#read_double) | `RmReadParameters < f64 >` | `f64` | **Rainmeter Only** |
| [read_formula](#read_formula) | `RmReadParameters < f64 >` | `f64` | **Rainmeter Only** |
| [read_int](#read_int) | `RmReadParameters < i32 >` | `i32` | **Rainmeter Only** |
| [read_string](#read_string) | `RmReadParameters < String >` | `String` | **Rainmeter Only** Read a string from Rainmeter. |

## execute_bang

**Signature:** `fn execute_bang(String) -> ()`

**Description:**  
**Rainmeter Only** Execute a Rainmeter Bang Example: [!SetVariable SomeVar 10]


## get_host

**Signature:** `fn get_host() -> String`


## get_skin_name

**Signature:** `fn get_skin_name() -> String`

**Description:**  
**Rainmeter Only**


## get_variable

**Signature:** `fn get_variable(String) -> String`

**Description:**  
**Rainmeter Only** Replace a Rainmeter Variable by its value var - The Var String, like: #MyVar#


## read_double

**Signature:** `fn read_double(RmReadParameters < f64 >) -> f64`

**Description:**  
**Rainmeter Only**


## read_formula

**Signature:** `fn read_formula(RmReadParameters < f64 >) -> f64`

**Description:**  
**Rainmeter Only**


## read_int

**Signature:** `fn read_int(RmReadParameters < i32 >) -> i32`

**Description:**  
**Rainmeter Only**


## read_string

**Signature:** `fn read_string(RmReadParameters < String >) -> String`

**Description:**  
**Rainmeter Only** Read a string from Rainmeter.

