set breakpoint pending on
break src/lib.rs:11  if idx == 4
# break sum_even if idx == values.length 
# delete 1
run
info args
info locals
bt
next
#panic