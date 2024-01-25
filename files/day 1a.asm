.data     

buffer:     .asciiz  "1abc2\npqr3stu8vwx\na1b2c3d4e5f\ntreb7uchet"

.text

    li $t0, 0 # Character index
    li $t2,  0  #Running sum
    li $t3, 10    # First digit (> 9 means not yet set for line)
    li $t4,  0#Second digit

    main:
lb $t1, buffer($t0)

bgt $t1, 64,    next_char  # If t1 is letter skip

beq $t1, '\n', end_line
beqz $t1, end

subi   $t4, $t1, 48          # Set second digit to number in t1

addi $t0,     $t0, 1

ble $t3, 9, main
subi $t3, $t1, 48#Set first digit to number in t1 if not set
j main

end_line:       
add $t2, $t2, $t4      
mul $t3,     $t3, 10
add $t2, $t2, $t3

li $t3, 10
li $t4, 0

addi $t0, $t0, 1
j    main

next_char:
addi $t0, $t0, 1
j main
	
end:
add $t2, $t2, $t4
mul $t3, $t3, 10
add $t2,     $t2, $t3

li $v0, 1
move $a0, $t2      
syscall

li $v0, 10
syscall



