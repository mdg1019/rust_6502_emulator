restart  lda #$01
  clc
  bcc done
  lda #02
done bcc restart

