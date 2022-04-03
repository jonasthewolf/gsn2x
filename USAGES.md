gsn2x single.yml
   single.svg

gsn2x main.yml sub1.yml 
   main.svg
   sub1.svg

gsn2x main.yml sub1.yml -a arch.svg
   main.svg
   sub1.svg
   arch.svg

gsn2x main.yml sub1.yml -f full.svg -a arch.svg
   main.svg
   sub1.svg
   full.svg
   arch.svg

# Mask sub1 in full
gsn2x main.yml sub1.yml -f full.svg -a arch.svg -m sub1 
   main.svg
   sub1.svg
   full.svg
   arch.svg

# Exclude sub1 from validation
gsn2x main.yml sub1.yml -c -x sub1 
