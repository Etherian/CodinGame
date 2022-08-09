I found a solution to my issue, and it's somewhat generalizable, which is what I was hoping for. The problem was that an implicit reference created in the `while let` statement was living to the end of the loop even though it was only needed on that one line. The borrow begins at `.iter()` and is no longer needed once the referenced value is `clone`d at the end of the expression.

````Rust
while let Some( (currid, _) ) = boundary.iter().max_by_key(|x| x.1).clone() {
    //                                  ^---where borrow begins    ^---where borrow could end

    // Move the node from `boundary` to `finished`
    let val = boundary.remove(&currid).unwrap();
    finished.insert(currid.clone(), val);

    ...

} // <--- where borrow does end
````

The trick was moving the binding of `currid` into the loop. When the value is borrowed in the `while let` statement, the borrow checker apparently thinks the borrow needs to last throughout the loop. If, instead, the implicit borrow is made in a regular `let` binding, the borrow checker is smart enough to realize the borrow can be safely discarded at the end of the line.

````Rust
while !boundary.is_empty() {
    let currid = boundary.iter().max_by_key(|x| x.1).unwrap().0.clone();
    //                   ^---where borrow begins               ^---where borrow ends
    // Move the node from `boundary` to `finished`
    let val = boundary.remove(&currid).unwrap();
    finished.insert(currid.clone(), val);

    ...

}
````

I guess the take away here is that if you need to mutate a structure in a loop that depends on it, put any borrows of the structure inside the loop and keep those borrows as short as possible – for example, by using `clone`.

This might be one of the situations eventually mitigated by the proposed [non-lexical lifetimes](https://github.com/nikomatsakis/nll-rfc/blob/master/0000-nonlexical-lifetimes.md).
