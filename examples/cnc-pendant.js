// Astargo Stratosphere Wired Number Pad for Laptop - Slim USB Numeric Keypad - Plug and Play 26 Keys
// https://www.amazon.com/dp/B0BPB63QC7

// Key Name
// Numlock On
// Numlock Off
// Fn+x (only effects first row)
//
// nl(xx)  num_lock (down/up) -> xx -> num_lock (down/up)
//
//  Calc       (        )         fn           bs
// la_app_2  alt+40   alt+41                backspace
// la_app_2  alt+40   alt+41                backspace
//  alt+36  alt+0128 alt+0165               backspace
//
// NumLock    esc       /         *            -
// numlock    esc    divide    multiply     subtract
// numlock    esc    divide    multiply    nl(subtract)
//
//    =        7        8         9            +
//  alt+61  numpad7  numpad8   numpad9        add
//  alt+61  nl(home) nl(up)    nl(page_up)    add
//
//   tab       4        5        6
//   tab    numpad4  numpad5   numpad6
//   tab    nl(left) nl(clear) nl(right)
//
//   00        1        2         3           enter
//   00     numpad1  numpad2   numpad3        enter
//   n/a    nl(end)  nl(down)  nl(page_down)  enter
//
//             0                decimal
//         nl(insert)          nl(delete)
//

let altState = 'up';
let altSequence = '';

rekeyRegister("PID_026C", "*", handleKeyEvent);

rekeyRegister("unknown", "*", handleKeyEvent);

/**
 * @param {KeyData} ctx 
 * @returns {boolean}
 */
function handleKeyEvent(ctx) {
  if (handleAltCodes(ctx)) {
    return true;
  }

  if (ctx.key) {
    handleKey(ctx.key);
  }
  return true;
}

/**
 * @param {KeyData} ctx 
 * @returns {boolean}
 */
function handleAltCodes(ctx) {
  if (ctx.vKeyCode == VK_ALT) {
    if (ctx.direction === 'up' && altSequence.length > 0) {
      switch (altSequence) {
        case '36': handleKey('$'); break;
        case '40': handleKey('('); break;
        case '41': handleKey(')'); break;
        case '61': handleKey('='); break;
        case '0128': handleKey('€'); break;
        case '0165': handleKey('¥'); break;
        default:
          console.error(`unhandled alt code ${altSequence}`);
          break;
      }
    } else if (ctx.direction === 'down') {
      altSequence = '';
    }
    altState = ctx.direction;
    return true;
  }

  if (ctx.ch >= '0' && ctx.ch <= '9') {
    if (altState === 'down') {
      altSequence += ctx.ch;
      return true;
    } else if (altSequence.length > 0) {
      const i = altSequence.indexOf(ctx.ch);
      if (i >= 0) {
        altSequence = altSequence.substring(0, i) + altSequence.substring(i + 1);
        return true;
      }
    }
  }

  altSequence = '';
  return false;
}

/**
 * @param {string} key
 * @returns {boolean}
 */
function handleKey(key) {
  switch (key) {
    // fn+calc
    case '$': break;

    // ( / €
    case '(': break;
    case '€': break;

    // ) / ¥
    case ')': break;
    case '¥': break;

    // backspace
    case 'backspace': break;

    // esc
    case 'esc': break;

    // /
    case 'divide': break;

    // *
    case 'multiply': break;

    // - / nl(subtract)
    case 'subtract':
      if (getKeyState(VK_NUM_LOCK).toggled) {
      } else {
      }
      break;

    // =
    case '=': break;

    // 7 / home
    case '7': case 'numpad7': break;
    case 'home': break;

    // 8 / up
    case '8': case 'numpad8': break;
    case 'up': break;

    // 9 / page_up
    case '9': case 'numpad9': break;
    case 'page_up': break;

    // add
    case 'add': break;

    // tab
    case 'tab': break;

    // 4 / left
    case '4': case 'numpad4': break;
    case 'left': break;

    // 5 / clear
    case '5': case 'numpad5': break;
    case 'clear': break;

    // 6 / right
    case '6': case 'numpad6': break;
    case 'right': break;

    // TODO 00

    // 1 / end
    case '1': case 'numpad1': break;
    case 'end': break;

    // 2 / down
    case '2': case 'numpad2': break;
    case 'down': break;

    // 3 / page_down
    case '3': case 'numpad3': break;
    case 'page_down': break;

    // enter
    case 'enter': break;

    // 0 / insert
    case '0': case 'numpad0': break;
    case 'insert': break;

    // decimal / delete
    case 'decimal': break;
    case 'delete': break;

    // ignore
    case 'num_lock': break;

    default:
      console.error(`unhandled key ${key}`);
      break;
  }
}
