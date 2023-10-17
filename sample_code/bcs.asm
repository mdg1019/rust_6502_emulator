restart  lda #$01
  sec
  bcs done
  lda #02
done bcs restart

