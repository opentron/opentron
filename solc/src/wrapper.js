var soljson = Module;

var version = soljson.cwrap('solidity_version', 'string', []);

var license = soljson.cwrap('solidity_license', 'string', []);

// >5.0.0
if ('_compileStandard' in soljson) {
    var compile = soljson.cwrap('compileStandard', 'string', ['string', 'number']);
} else {
    var compile = soljson.cwrap('solidity_compile', 'string', ['string', 'number']);
}
