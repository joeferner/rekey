/**
 * ReKey API
 */

/**
 * Register a callback for each key event.
 * 
 * @global
 * @function
 * @name rekeyRegister
 * @param {RegisterOptions} options Options to filter keys
 * @param {keyCallback} callback Callback to be called on each key event
 */
function rekeyRegister(options, callback) { }

/**
 * Send a key event
 * 
 * @global
 * @function
 * @name sendKey
 * @param {string} keyExpression The key expression to send. Examples: 'ctrl+esc', 'a', 'alt+f4'
 * @param {'up'|'down'} [direction] If specified only send the given key direction, otherwise send both down
 *                                  and up events.
 */
function sendKey(keyExpression, direction) { }

/**
 * Get the state of a key
 * 
 * @global
 * @function
 * @name getKeyState
 * @param {number} vKeyCode The virtual key code
 * @returns {GetKeyStateResult}
 */
function getKeyState(vKeyCode) { }

/**
 * Data passed to the rekeyRegister callback.
 * 
 * @typedef {Object} KeyEvent
 * @property {number} vKeyCode Virtual key code
 * @property {string} [key] String representation of the key.
 * @property {string} [ch] String representation of the key.
 * @property {'up'|'down'} direction The direction of the key event
 * @property {string} [deviceName] The device name from which the event was generated.
 */

/**
 * @typedef {Object} RegisterOptions
 * @param {string} [deviceFilter] If present no device filtering will be done. If a string is passed and the
 *                                  device name contains that string the callback will be called.
 * @param {'*'} [keyFilter] Currently must be '*' and no key filtering will be done.
 * @param {boolean} [intercept] If true the result of the callback can effect if the key gets passed on to the system.
 */

/**
 * @callback keyCallback
 * @param {KeyEvent} event Data about the key press
 * @returns {boolean} true, if the keyboard event should be filtered. false, if the keyboard event should not be filterd.
 */

/**
 * The results from getKeyState
 * 
 * @typedef {Object} GetKeyStateResult
 * @property {'down'|'up'} state The current pressed state
 * @property {boolean} toggled True if the key is toggled on i.e. caps lock
 */

