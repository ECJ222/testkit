# Testkit Testing Tool Documentation

## Introduction

Testkit is a testing tool designed for API manual testing and test automation tasks. It provides a simplified yaml syntax for defining and executing API test scenarios.
<img width="1107" alt="image" src="https://github.com/apitoolkit/testkit/assets/6564482/d1f7ec76-b81b-4036-a87b-b8bda562d03c">

### Why a testing DSL?

- Teams should not be forced to write javascript (Postman), Groovy (Katalon), or Java (Rest-Assured) just because they want to test an API for a web system
- We should be able to create interactive builders that generate this underlying DSL. So you shouldn't even need to write this DSL by hand in the future
- We should be able to use the same script for both individual tests and load testing.
- We should still be able to persist these tests in our version control and collaborate on them with our peers
- Inspired by [Local-First software](https://www.inkandswitch.com/local-first/) principles

## Table of Contents

- [Installation](#installation)
- [How to Run](#how-to-run)
- [Test Definition Syntax](#test-definition-syntax)
- [Comparison with Other Testing Libraries](#comparison-with-other-testing-libraries)
- [What is JSONPath](#what-is-jsonpath)
- [The `request` Field](#request-field)
- [The `asserts` Field](#asserts-field)
- [The `outputs` Field](#outputs)
- [Referencing Values and Dynamic Inputs for Subsequent API Requests](#referencing-values-and-dynamic-inputs-for-subsequent-api-requests)
- [Using environment variables](#using-environment-variables)

FYI, this table of contents reflects the current sections covered in the documentation based on the information provided so far. It may be expanded or revised as the documentation progresses and more content is added.

## Installation

To install the `testkit` testing tool, follow the steps below:

1. Ensure you have Rust programming language and Cargo package manager installed on your system. You can download them from the official Rust website: [https://www.rust-lang.org/](https://www.rust-lang.org/).

2. Clone the `testkit` repository from GitHub using the following command:

   ```
   git clone https://github.com/`testkit`/`testkit`.git
   ```

3. Change into the cloned directory:

   ```
   cd `testkit`
   ```

4. Build the project using Cargo:

   ```
   cargo build --release
   ```

5. Once the build process is complete, you can find the `testkit` executable file in the `target/release` directory.

## How to Run

To run the `testkit` testing tool, use the following command:

```bash
RUST_LOG=debug cargo run -- --file ./test.yaml
```

Make sure to replace `./test.yaml` with the path to your YAML test file.

## Comparison with Other Testing Libraries

Before delving into the details of `testkit` and its functionality, let's compare it to other testing libraries to highlight the elegance, cleanliness, and simplicity of `testkit`. To illustrate this, we will rewrite a Cypress test using `testkit`.

By showcasing the differences, you will see how `testkit` provides a more streamlined and user-friendly approach to API testing. It offers a clean and intuitive syntax that simplifies the process of defining and executing API test scenarios. Let's explore the rewritten test using `testkit`.

```js
describe('TODO api testing', () => {
  let todoItem;
  it('fetches Todo items - GET', () => {
    cy.request('/todos/').as('todoRequest');
    cy.get('@todoRequest').then((todos) => {
      todoItem = todos.body[0]['_id'];
      expect(todos.status).to.eq(200);
      assert.isArray(todos.body, 'Todos Response is an array');
    });
  });

  it('deletes Todo items - DELETE', () => {
    cy.request('DELETE', `/todos/${todoItem}`).as('todoRequest');
    cy.get('@todoRequest').then((todos) => {
      expect(todos.status).to.eq(200);
      assert.isString(todos.body, 'todo deleted!');
    });
  });

  it('Adds Todo item - POST', () => {
    cy.request('POST', '/todos/', { task: 'run tests' }).as('todoRequest');
    // adds new Todo item by defining Todo name
    cy.get('@todoRequest').then((todos) => {
      expect(todos.status).to.eq(200);
      cy.wrap(todos.body).should('deep.include', {
        task: 'run tests',
        completed: false,
      });
    });
  });
});
```

```yaml
---
- name: TODO api testing
  stages:
    - name: fetches TODO items - GET
      GET: /todos/
      asserts: # Asserts accepts a list of expressions, usually via json_paths to identify the items being refered to.
        ok: $.resp.status_code == 200  # Rely on an expressions libray for parsing expressions
        array: $.resp.body.json
      outputs: # values which should be accesible to future steps.
        todoItem: $.resp.body.json[0]._id

    - name: deletes TODO items - DELETE
      DELETE: /todos/$.stages[0].outputs.todoItem # relative syntax exists: $.stages[-1].outputs.todoItem, -1 means one stage before me
      asserts:
        empty: $.resp.body.json.todos
        string: $.resp.body.json

    - name: Adds Todo item - POST
      POST: /todos/
      json:
        task: "run tests"
      asserts:
        ok: $.resp.status_code == 200
        ok: $.resp.body.json.task == "run tests"
        ok: $.resp.body.json.completed == false
```

## Test Definition Syntax

`testkit` uses a YAML-based syntax for defining test scenarios. Each scenario consists of multiple stages, where each stage represents a specific API request and its associated assertions. Below is an example of the YAML syntax for defining API tests:

```yaml
---
- name: TODO api testing
  stages:
    - name: fetches TODO items - GET
      GET: /todos/
      asserts:
        ok: $.resp.status_code == 200
        array: $.resp.body.json
      outputs:
        todoItem: $.resp.body.json[0]._id

    - name: deletes TODO items - DELETE
      DELETE: /todos/$.stages[0].outputs.todoItem
      asserts:
        empty: $.resp.body.json.todos
        string: $.resp.body.json

    - name: Adds Todo item - POST
      POST: /todos/
      json:
          task: "run tests"
      asserts:
        ok: $.resp.status_code == 200
        ok: $.resp.body.json.task == "run tests"
        ok: $.resp.body.json.completed
```

The YAML file consists of a list of test scenarios. Each scenario contains a `name` field and a list of `stages`. Each stage represents an API request and contains the following fields:

- `name` (required): A descriptive name for the stage.
- `request` (required): Defines the API request to be made. It can include HTTP methods (`GET`, `POST`, `PUT`, `DELETE`, etc.) and the corresponding request URL or endpoint.
- `asserts` (optional): Defines the assertions to be performed on the response. It specifies conditions that must be satisfied for the test to pass.
- `outputs` (optional): Specifies the values to be captured from the response and made available to future stages.

In the example above, the YAML test file defines three stages: fetching TODO items using a GET request, deleting a specific TODO item using a DELETE request, and adding a new TODO item using a POST request.
The `name` field is self explanatory and so we'll take more about the rest of the fields in detail but before that let's talk about JSONPath.

### What is JSONPath

JSONPath is a powerful query language designed for navigating and extracting data from JSON documents. It provides a concise syntax that allows you to specify paths to specific elements within a JSON structure, facilitating data access and manipulation. In `testkit`, JSONPath expressions are extensively used to extract data for assertions and outputs.

To illustrate how JSONPath works, consider the following examples:

- `$.user.name`: This expression retrieves the name of a user from the top-level object in the JSON document.
- `$.todos[0].task`: Here, the expression accesses the task property of the first element in an array of todos.
- `$.todos[*].task.description`: This expression retrieves the description property of all tasks within the todos array.

The syntax of JSONPath expressions includes several key components:

- Bracket notation (`[]`): Used to access elements within an array by providing the index within square brackets.
- Wildcard (`*`): Matches any element at the current level, allowing you to retrieve all elements of a particular level.
- Recursive descent (`..`): Enables searching for elements at any depth within the JSON structure, including nested objects and arrays.
- Filters (`[?]`): Allows applying conditions or filters to select specific elements based on certain criteria.

By employing JSONPath expressions, you can precisely pinpoint the desired data within a JSON structure. These expressions play a vital role in `testkit`, facilitating the extraction of data for performing assertions and capturing outputs during the testing process. learn more about jsonpaths [here](https://lzone.de/cheat-sheet/JSONPath)

### request field

The `request` field in `testkit` defines the API request to be made and consists of three properties:

- `method`: This property specifies the HTTP method for the request, such as `GET`, `POST`, `PUT`, or `DELETE`. The value of the `method` property is the request URL path.

  Example:

  ```yaml
  # POST request
  - name: Adds Todo item - POST
    POST: /todos/

  # GET request
  - name: Fetches Todo items - GET
    GET: /todos/
  ```

- `headers` (optional): This property allows you to include HTTP headers in the request. Headers can be used to pass additional information to the server, such as authentication tokens or content type.

  Example:

  ```yaml
  - name: Fetches Todo items - GET with headers
    GET: /todos/
    headers:
      Authorization: Bearer <token>
      Content-Type: application/json
      Allowed-Methods:
        - GET
        - POST
  ```

- `json` The `request` field in `testkit` also supports an optional property called `json`. This property allows you to include request body data in JSON format when necessary.
  By specifying the `json` property within the `request` field, you can provide structured data that needs to be sent along with the API request.
  Here's an example illustrating the usage of the `json` property:

  ```yaml
  - name: Create User - POST
    POST: /users/
    json:
      name: John Doe
      age: 25
      email: john.doe@example.com
  ```

  In the above example, a POST request is made to create a new user. The `json` property contains the user data in JSON format, including properties such as `name`, `age`, and `email`.

  Including the `json` property in the `request` field enables you to pass structured data to the API endpoint, facilitating actions such as creating or updating resources on the server.

These properties in the `request` field provide flexibility and control over the API requests made during testing. You can specify the HTTP method and include headers as needed to interact with the API endpoints effectively.

## asserts field

The `asserts` field in `testkit` plays a crucial role in defining assertions or validations to be performed on the API response. It allows you to specify conditions that must be met for the test to pass successfully.

The `asserts` field accepts a collection of key-value pairs, where the keys represent the type of assertion and the values define the corresponding expressions or conditions to be evaluated.

Here's an example to demonstrate the usage of the `asserts` field:

```yaml
- name: Fetches Todo items - GET
  GET: /todos/
  asserts:
    ok: $.resp.status_code == 200
    array: $.resp.body.json
    ok: $.resp.body.json[0].task == "run tests"
```

The `.json` tells `testkit` to convert the response into JSON format.
This allows you to access properties of the response JSON using JSONPath expressions.

In the above example, we have defined three assertions:

1. `ok`: This assertion checks whether the response status code is equal to 200. The expression `$.resp.status_code == 200` is evaluated, and if it returns `true`, the assertion is considered successful.

2. `array`: This assertion verifies that the response body is an array. The expression `$.resp.body.json` is evaluated, and if the result is an array, the assertion passes.

You can include multiple assertions within the `asserts` field to perform various validations on different aspects of the API response, such as checking specific properties, verifying the presence of certain data, or comparing values.

By utilizing the `asserts` field effectively, you can ensure that the API response meets the expected criteria, providing confidence in the correctness and reliability of your API.
All possible assertions you could use in the `asserts` field of `testkit` are as follows:

- `ok`: Checks if the provided expression evaluates to `true`.
- `empty`: Checks if a value is empty (e.g., an empty array, string, or null).
- `array`: Checks if a value is an array.
- `string`: Checks if a value is a string.
- `number`: Checks if a value is a number.
- `boolean`: Checks if a value is a boolean.
- `null`: Checks if a value is null.

These assertions provide a wide range of options to validate different aspects of the API response, allowing you to ensure the correctness and integrity of the data and behavior. You can select the appropriate assertion based on the specific validation requirements of your API test scenario.

## outputs

The `outputs` field in `testkit` allows you to capture and store values from the API response of a stage for future reference within the test scenario. It provides a convenient way to extract specific data and make it accessible in subsequent stages of the test.

To use the `outputs` field, you define key-value pairs where the keys represent the names of the outputs (think of it as a variable), and the values define the JSON paths or expressions used to extract the desired data from the response.

Here's an example that demonstrates the usage of the `outputs` field:

```yaml
- name: Fetches Todo items - GET
  GET: /todos/
  outputs:
    todoItem: $.resp.body.json[0]._id
```

In the above example, the `outputs` field captures the value of the `_id` property from the first element of the API response array. It assigns this value to the `todoItem` output.

By capturing the `_id` value in the `todoItem` output, you can access it in subsequent stages of the test scenario. This allows you to use the extracted data for further API requests, assertions, or any other necessary operations.

The `outputs` field enables you to create a bridge between different stages within the test scenario, providing a way to pass relevant data between them. This can be particularly useful when you need to refer to specific values or dynamically generate inputs for subsequent API requests.

Using the `outputs` field, you can enhance the flexibility and modularity of your API tests, making them more robust and adaptable to different scenarios.

## Referencing Values and Dynamic Inputs for Subsequent API Requests

The `outputs` field in `testkit` not only allows you to capture values from the API response but also provides a powerful mechanism for referencing those values and dynamically generating inputs for subsequent API requests.

By capturing relevant data using the `outputs` field, you can store it as an output and easily refer to it in later stages of your test scenario. This capability becomes particularly useful when you need to access specific values extracted from the response and utilize them in subsequent API requests.

For example, let's say you retrieve an ID from an API response in one stage using the `outputs` field:

```yaml
- name: Fetch User - GET
  GET: /users/1
  outputs:
    userId: $.resp.body.id
```

To reference this `userId` output in a subsequent API request, you can use the `$.stages[n].outputs` syntax:

```yaml
- name: Update User - PUT
  PUT: /users/$.stages[0].outputs.userId
  json:
    name: 'John Doe'
```

In the above example, the `userId` captured in the first stage is accessed using the syntax `$.stages[0].outputs.userId`. `testkit` understands that it should substitute the reference with the corresponding value during execution.

You can also use relative references like `$.stages[-n]` which refers to the output of the `nth` stage before the current stage.
Example:

```yaml
- name: deletes TODO items - DELETE
  DELETE: /todos/$.stages[-1].outputs.todoItem #-1 means one stage before me
  asserts:
    string: $.resp.body.json.task
    ok: $.resp.body.json.id == $.stages[-1].outputs.todoItem
```

By referencing specific values captured in previous stages, you can establish dependencies between different API requests and ensure seamless data flow throughout your test scenario. This flexibility allows you to build more comprehensive and realistic tests, simulating complex user interactions or workflows.

## Using environment variables

Testkit supports environment variables in two ways: using a `.env` file or directly setting environment variables. These approaches allow users to configure and customize their test scripts without exposing sensitive data and making it easier to switch between different environments and scenarios seamlessly. Here's how each method works:

Using a `.env` file involves creating a text file named `.env` in the test script's directory and defining `KEY=VALUE` pairs for each environment variable. Testkit automatically loads these variables from the `.env` file during test execution.

Example `.env` file:

```shell
APIURL=https://api.example.com
EMAIL=user@example.com
PASSWORD=mysecretpassword
USERNAME=myusername
APIKEY=mysecretapikey
```

Setting environment variables directly is done via the command-line or the test environment.

Example command-line usage:

```shell
APIKEY=SECRETAPIKEY cargo run -- --file test.yaml
```

To utilize environment variables in Testkit, you can access them using the following syntax: `$.env.<VAL>`, where `<VAL>` represents the name of the specific environment variable you want to use. This allows you to easily reference and incorporate the values of these environment variables within your test scripts, enabling greater flexibility and adaptability without hardcoding sensitive information or configuration details.

Example:

```yaml
- name: Register
  POST: '$.env.APIURL/users'
  headers:
    Content-Type: application/json
    X-Requested-With: XMLHttpRequest
  json: '{"user":{"email":"$.env.EMAIL", "password":"$.env.PASSWORD", "username":"$.env.USERNAME"}}'
  asserts:
    exists: $.resp.json.user
    exists: $.resp.json.user.email
    exists: $.resp.json.user.username
    exists: $.resp.json.user.bio
    exists: $.resp.json.user.image
    exists: $.resp.json.user.token
```

In this example, Testkit performs a POST request to the API URL specified in the environment variable `APIURL`. The user information for registration is taken from the environment variables `EMAIL`, `PASSWORD`, and `USERNAME`, allowing for easy customization and reusability of the test script across different environments.
