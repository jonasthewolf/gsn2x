gsn2x single.yml
   single.dot

gsn2x single.yml -o
   <stdout>

gsn2x main.yml sub1.yml 
   main.dot
   sub1.dot

gsn2x main.yml sub1.yml -a arch.dot
   main.dot
   sub1.dot
   arch.dot

gsn2x main.yml sub1.yml -f full.dot -a arch.dot
   main.dot
   sub1.dot
   full.dot
   arch.dot

# Mask sub1 in full
gsn2x main.yml sub1.yml -f full.dot -a arch.dot -m sub1 
   main.dot
   sub1.dot
   full.dot
   arch.dot

# Exclude sub1 from validation
gsn2x main.yml sub1.yml -c -x sub1 
