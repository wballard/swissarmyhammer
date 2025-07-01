---
name: test-property
title: Generate Property-Based Tests
description: Create property-based tests to find edge cases automatically
arguments:
  - name: code
    description: The code to test with properties
    required: true
  - name: framework
    description: Property testing framework (quickcheck, hypothesis, proptest, etc.)
    required: false
    default: "auto-detect"
  - name: properties_to_test
    description: Specific properties or invariants to verify
    required: false
    default: "common properties"
  - name: num_examples
    description: Number of random examples to generate
    required: false
    default: "100"
---

# Property-Based Testing

## Code Under Test
```
{{{code}}}
```

## Test Configuration
- **Framework**: {{framework}}
- **Properties**: {{properties_to_test}}
- **Examples**: {{num_examples}}

## Property Testing Strategy

### 1. Identify Properties

#### Invariants
- Conditions that always hold
- Mathematical properties
- Business rules
- Data constraints

#### Relationships
- Input/output relationships
- Inverse operations
- Idempotent operations
- Commutative properties

#### Edge Cases
- Boundary values
- Empty collections
- Null/undefined handling
- Overflow conditions

### 2. Property Categories

#### Algebraic Properties
- **Associativity**: (a + b) + c = a + (b + c)
- **Commutativity**: a + b = b + a
- **Identity**: a + 0 = a
- **Inverse**: a + (-a) = 0

#### Functional Properties
- **Idempotence**: f(f(x)) = f(x)
- **Homomorphism**: f(a ∙ b) = f(a) ∙ f(b)
- **Monotonicity**: a ≤ b → f(a) ≤ f(b)

#### Structural Properties
- **Size preservation**: len(filter(xs)) ≤ len(xs)
- **Element preservation**: all elements in output exist in input
- **Order preservation**: sorted remains sorted

### 3. Generator Strategies

#### Basic Generators
- Primitives: integers, floats, strings
- Collections: lists, sets, maps
- Structured: objects, tuples

#### Custom Generators
- Domain-specific values
- Constrained inputs
- Weighted distributions
- Shrinking strategies

#### Composition
- Combining generators
- Dependent generators
- Recursive structures
- State machines

### 4. Property Implementation

```{{framework}}
// Example property test structure
property("{{properties_to_test}}", {
  // Generate random inputs
  forAll(generator, (input) => {
    // Execute function
    const result = functionUnderTest(input);
    
    // Assert properties
    return checkProperty(input, result);
  });
});
```

### 5. Common Patterns

#### Round-trip Properties
```
decode(encode(x)) === x
parse(toString(x)) === x
```

#### Invariant Properties
```
sort(xs).length === xs.length
reverse(reverse(xs)) === xs
```

#### Model-based Properties
```
fastImplementation(x) === referenceImplementation(x)
optimized(x) === naive(x)
```

### 6. Failure Analysis

When properties fail:
- Examine counterexamples
- Understand failure patterns
- Add regression tests
- Refine properties
- Fix implementation

### 7. Best Practices
- Start with simple properties
- Use shrinking for minimal examples
- Combine with example-based tests
- Document property meanings
- Monitor test performance