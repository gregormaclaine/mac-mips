
.data     
output:     .asciiz  "Hello World"

.text


    main:

li $v0, 4 #Prepares to print string
la $a0,   output          #     Loads address of the output
syscall # Prints the string
 end:  
    li $v0   , 10
    syscall

