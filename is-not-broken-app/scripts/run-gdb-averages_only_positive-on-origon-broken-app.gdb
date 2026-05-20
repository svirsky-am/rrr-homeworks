set breakpoint pending on
#break average_positive
#break averages_only_positive
#next
break src/lib.rs:52
run
#break src/lib.rs:49 #
# break sum_even if idx == values.length 
# delete 1
#run
#info args
#info locals
#bt
#next
#panic