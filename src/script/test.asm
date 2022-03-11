; a comment to start
start:   ; useless label, jor for test
        lda #$c0 ; A9 C0
                 ;load A from 0xC0
        tax      ; AA
                 ;transfer A to X 
        inx      ; e8
                 ; Increment X
        brk      ; 00
                 ; break
end: ; end of the program