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
let doubleZeroTimeout = undefined;

rekeyRegister({ deviceFilter: "PID_026C", intercept: true }, handleKeyEvent);

/**
 * @param {KeyEvent} event
 * @returns {boolean}
 */
function handleKeyEvent(event) {
  if (handleAltCodes(event)) {
    return true;
  }

  if (event.key && event.direction === 'down') {
    if (event.ch === '0') {
      // timeout didn't occur so we must have gotten a very quick
      // '0' then '0' which comes from '00' key
      if (doubleZeroTimeout) {
        clearTimeout(doubleZeroTimeout);
        doubleZeroTimeout = undefined;
        handleKey('00', event);
      } else {
        // could be a '00' key so set a quick timeout. If it occurs
        // it must be the '0' key held down.
        doubleZeroTimeout = setTimeout(() => {
          doubleZeroTimeout = undefined;
          handleKey('0', event);
        }, 20);
      }
    } else {
      handleKey(event.key, event);
    }
  }
  return true;
}

/**
 * @param {KeyEvent} event
 * @returns {boolean}
 */
function handleAltCodes(event) {
  if (event.vKeyCode == VK_ALT) {
    if (event.direction === 'up' && altSequence.length > 0) {
      switch (altSequence) {
        case '36':
          handleKey('$', event);
          break;
        case '40':
          handleKey('(', event);
          break;
        case '41':
          handleKey(')', event);
          break;
        case '61':
          handleKey('=', event);
          break;
        case '0128':
          handleKey('€', event);
          break;
        case '0165':
          handleKey('¥', event);
          break;
        default:
          console.error(`unhandled alt code ${altSequence}`);
          break;
      }
    } else if (event.direction === 'down') {
      altSequence = '';
    }
    altState = event.direction;
    return true;
  }

  if (event.ch >= '0' && event.ch <= '9') {
    if (altState === 'down') {
      altSequence += event.ch;
      return true;
    } else if (altSequence.length > 0) {
      const i = altSequence.indexOf(event.ch);
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
 * @param {KeyEvent} event
 * @returns {boolean}
 */
function handleKey(key, event) {
  switch (key) {
    // fn+calc
    case '$':
      console.log('$');
      break;

    // ( / €
    case '(':
      console.log('(');
      break;
    case '€':
      console.log('€');
      break;

    // ) / ¥
    case ')':
      console.log(')');
      break;
    case '¥':
      console.log('¥');
      break;

    // backspace
    case 'backspace':
      console.log('backspace');
      break;

    // esc
    case 'esc':
      console.log('esc');
      break;

    // /
    case 'divide':
      console.log('divide');
      break;

    // *
    case 'multiply':
      console.log('multiply');
      break;

    // - / nl(subtract)
    case 'subtract':
      if (getKeyState(VK_NUM_LOCK).toggled) {
        console.log('subtract');
      } else {
        console.log('nl(subtract)');
      }
      break;

    // =
    case '=':
      console.log('=');
      break;

    // 7 / home
    case '7':
    case 'numpad7':
      console.log('7');
      break;
    case 'home':
      console.log('home');
      break;

    // 8 / up
    case '8':
    case 'numpad8':
      console.log('8');
      break;
    case 'up':
      console.log('up');
      break;

    // 9 / page_up
    case '9':
    case 'numpad9':
      console.log('9');
      break;
    case 'page_up':
      console.log('page up');
      break;

    // add
    case 'add':
      console.log('add');
      break;

    // tab
    case 'tab':
      console.log('tab');
      break;

    // 4 / left
    case '4':
    case 'numpad4':
      console.log('4');
      break;
    case 'left':
      console.log('left');
      break;

    // 5 / clear
    case '5':
    case 'numpad5':
      console.log('5');
      break;
    case 'clear':
      console.log('clear');
      break;

    // 6 / right
    case '6':
    case 'numpad6':
      console.log('6');
      break;
    case 'right':
      console.log('right');
      break;

    // 00
    case '00':
      console.log('00');
      break;

    // 1 / end
    case '1':
    case 'numpad1':
      console.log('1');
      break;
    case 'end':
      console.log('end');
      break;

    // 2 / down
    case '2':
    case 'numpad2':
      console.log('2');
      break;
    case 'down':
      console.log('down');
      break;

    // 3 / page_down
    case '3':
    case 'numpad3':
      console.log('3');
      break;
    case 'page_down':
      console.log('page down');
      break;

    // enter
    case 'enter':
      console.log('enter');
      break;

    // 0 / insert
    case '0':
    case 'numpad0':
      console.log('0');
      break;
    case 'insert':
      console.log('insert');
      break;

    // decimal / delete
    case 'decimal':
      console.log('.');
      break;
    case 'delete':
      console.log('delete');
      break;

    // ignore
    case 'num_lock':
      break;

    default:
      console.error(`unhandled key ${key}: ${JSON.stringify(event)}`);
      break;
  }
}
