
rekeyRegister("PID_026C", "*", ctx => {
  if (ctx.vKeyCode == 144) {
	  sendKey("ctrl+esc", ctx.direction);
  }
  return false;
});
