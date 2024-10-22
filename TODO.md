# TODO

## Parser

- [x] Fix errors when compiling expressions containing nested unary operators. (`~(-(3))`) 

## Assembly 

- [ ] Add comments to assembly code. 
- [ ] Implement visitor patter to carry register allocation, instruction modification, etc. 

## CLI 

- [ ] Modify CLI arguments to implement compiler passes on various levels. 

## Code emission 

- [ ] Design a better test suite to check correctness of assembly generation for nested binary operators. 
- [ ] Write a test for `playground/test_add_mul.c` and make it pass. 

## End-to-end

- [ ] Implement End-to-end test suite that check correctness of the gcc compiler output.
