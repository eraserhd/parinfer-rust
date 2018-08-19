var wasm = require(__dirname + '/parinfer_rust.js');

function mode(mode) {
  return function(text, options) {
    return JSON.parse(wasm.run_parinfer(JSON.stringify({
      mode: mode,
      text: text,
      options: options
    })));
  };
}

module.exports = {
  run_parinfer: wasm.run_parinfer,
  indentMode: mode('indent'),
  parenMode: mode('paren'),
  smartMode: mode('smart')
};
