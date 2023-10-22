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
let sentCtrlAltDown = false;

rekeyRegister({ deviceFilter: "PID_026C", intercept: true }, handleKeyEvent);

/**
 * @param {string} ch
 * @param {KeyData} event
 */
function sendCtrlAlt(ch, event) {
  if (event.direction === 'down' && !sentCtrlAltDown) {
    sendKey('ctrl+alt', 'down');
    sentCtrlAltDown = true;
  }
  sendKey(ch, event.direction);
  if (event.direction === 'up' && sentCtrlAltDown) {
    sendKey('ctrl+alt', 'up');
    sentCtrlAltDown = false;
  }
}

/**
 * @param {KeyData} event
 * @returns {boolean}
 */
function handleKeyEvent(event) {
  if (handleAltCodes(event)) {
    return true;
  }

  if (handleDoubleZero(event)) {
    return true;
  }

  handleKey(event.key, event);
  return true;
}

/**
 * @param {KeyData} event
 * @returns {boolean} true, if handled
 */
function handleDoubleZero(event) {
  if (event.ch !== '0') {
    return false;
  }

  if (event.key && event.direction === 'down') {
    // timeout didn't occur so we must have gotten a very quick
    // '0' then '0' which comes from '00' key
    if (doubleZeroTimeout) {
      clearTimeout(doubleZeroTimeout);
      doubleZeroTimeout = undefined;
      handleKey('00');
    } else {
      // could be a '00' key so set a quick timeout. If it occurs
      // it must be the '0' key held down.
      doubleZeroTimeout = setTimeout(() => {
        doubleZeroTimeout = undefined;
        handleKey('0');
      }, 20);
    }
  }
  return true;
}

/**
 * @param {KeyData} event
 * @returns {boolean} true, if handled
 */
function handleAltCodes(event) {
  if (event.vKeyCode == VK_ALT) {
    if (event.direction === 'up' && altSequence.length > 0) {
      switch (altSequence) {
        case '36':
          handleKey('$');
          break;
        case '40':
          handleKey('(');
          break;
        case '41':
          handleKey(')');
          break;
        case '61':
          handleKey('=');
          break;
        case '0128':
          handleKey('€');
          break;
        case '0165':
          handleKey('¥');
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
 * @param {KeyData} [event]
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
      // Send: Y+
      sendCtrlAlt('w', event);
      break;

    // 9 / page_up
    case '9':
    case 'numpad9':
      console.log('9');
      break;
    case 'page_up':
      // Send: Z+
      sendCtrlAlt('r', event);
      break;

    // add
    case 'add':
      if (event.direction === 'up') {
        // Send: Multiply XY Step Size by 10
        sendKey('ctrl+alt+t');
        // Send: Multiply Z Step Size by 10
        sendKey('ctrl+alt+y');
      }
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
      // Send: X-
      sendCtrlAlt('a', event);
      break;

    // 5 / clear
    case '5':
    case 'numpad5':
      console.log('5');
      break;
    case 'clear':
      // Send: Home
      sendCtrlAlt('q', event);
      break;

    // 6 / right
    case '6':
    case 'numpad6':
      console.log('6');
      break;
    case 'right':
      // Send: X+
      sendCtrlAlt('d', event);
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
      // Send: Y-
      sendCtrlAlt('s', event);
      break;

    // 3 / page_down
    case '3':
    case 'numpad3':
      console.log('3');
      break;
    case 'page_down':
      // Send: Z-
      sendCtrlAlt('f', event);
      break;

    // enter
    case 'enter':
      if (event.direction === 'up') {
        // Send: Divide XY Step Size by 10
        sendKey('ctrl+alt+g');
        // Send: Divide Z Step Size by 10
        sendKey('ctrl+alt+h');
      }
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
