; Offset of current instruction
zp_byte instrs_idx

zp_byte failed_count
zp_byte interrupt_occurred

.ifdef OUR_CUSTOM_IRQ
irq:
	pla
	pha
	and #$10
	beq :+
	ldx #$A2        ; fix stack
	txs
	jsr reset_crc ; force failure
	jmp force_failure
.endif
nmi:    bit interrupt_occurred
	bpl @interrupt
	bit $4015
	rti
@interrupt:
	dec interrupt_occurred
	mov PPUCTRL,#0
	mov SNDMODE,#$C0
	set_test 2,"No interrupts should occur during test"
	jmp test_failed

main:
	; Stack slightly lower than top
	ldx #$A2
	txs
	
	jsr init_crc_fast
	
	; Test each instruction
	lda #0
instr_loop:
	sta instrs_idx
	tay
	
	jsr reset_crc
	lda instrs,y
	jsr test_instr
force_failure:
	jsr check_result
	
	lda instrs_idx
	clc
	adc #4
	cmp #instrs_size
	bne instr_loop
	
.ifdef BUILD_DEVCART
	lda #0
	jmp exit
.endif

	lda failed_count
	jne test_failed
	jmp tests_passed

; Check result of test
check_result:
.ifdef BUILD_DEVCART
	; Print correct CRC
	jsr crc_off
	print_str ".dword $"
	ldx #0
:       lda checksum,x
	jsr print_hex
	inx
	cpx #4
	bne :-
	jsr print_newline
	jsr crc_on
.else
	; Verify CRC
	ldx #3
	ldy instrs_idx
:       lda checksum,x
	cmp correct_checksums,y
	bne @wrong
	iny
	dex
	bpl :-
.endif
	rts

; Print failed opcode and name
@wrong: 
	ldy instrs_idx
	lda instrs,y
	jsr print_a
	jsr play_byte
	lda instrs+2,y
	sta addr
	lda instrs+3,y
	sta addr+1
	jsr print_str_addr
	jsr print_newline
	inc failed_count
	rts

; Place where instruction is executed
instr = $3A0

; Tests instr A
test_instr:
	sta instr
	jsr avoid_silent_nsf
	
	; Copy rest of template
	ldx #instr_template_size - 1
:       lda instr_template,x
	sta instr,x
	dex
	bne :-
	
	; Disable and be sure APU IRQs are clear, since
	; I flag gets cleared during testing.
	setb SNDMODE,$C0 
	setb $4010,0
	nop
	lda SNDCHN
	
	; Default stack
	lda #$90
	sta in_s
	
	; Test with different flags
	lda #$00
	jsr test_flags
	lda #$FF
	jsr test_flags
	
	rts

; Position in operand table
zp_byte operand_idx

test_flags:
	sta in_p
	
	ldy #values_size-1
:       sty operand_idx

	lda values,y
	sta in_a
	
	lda values+1,y
	sta in_x
	
	lda values+2,y
	sta in_y
	
	jsr test_values
	
	ldy operand_idx
	dey
	bpl :-
	
	rts

.ifndef values2
	values2 = values
	values2_size = values_size
.endif

.macro test_normal
zp_byte a_idx
zp_byte saved_s
	
	tsx
	stx saved_s
	
	set_stack
	
	ldy #values2_size-1
inner:  sty a_idx
	
	lda values2,y
	sta operand
	
	set_in

; For debugging 
.if 0
	; P A X Y S O (z,x) (z),y
	jsr print_p
	jsr print_a
	jsr print_x
	jsr print_y
	jsr print_s
	lda operand
	jsr print_a
.ifdef address
	lda (address,x)
	jsr print_a
	lda (address),y
	jsr print_a
.else
	lda operand,x
	jsr print_a
	lda operand,y
	jsr print_a
.endif
	jsr print_newline
.endif

	jmp instr
instr_done:
	
	check_out
	
	ldy a_idx
	dey
	bpl inner
	
	check_stack
	
	ldx saved_s
	txs
.endmacro
