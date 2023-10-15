
rekeyRegister("PID_026C", "*", ctx => {
  if (ctx.vKeyCode == VK_1) {
	  sendKey("ctrl+esc", ctx.direction);
  }
  return true;
});

rekeyRegister("unknown", "*", ctx => {
  console.log(ctx.vKeyCode);
  return true;
});
