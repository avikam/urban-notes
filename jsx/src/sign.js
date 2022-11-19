import sha from "sha.js";

export function sign(tokenAccountName, password, query) {
    const nonce = makeNonce(10);
    const timestamp = Math.round(Date.now() / 1000);
    const account = fixedEncodeURIComponent(tokenAccountName);
    const buf = sha('sha256').update(`${tokenAccountName}_${password}_${query}_${nonce}_${timestamp}`);

    const signature = buf.digest('base64');
    return {
        signature,
        nonce, 
        timestamp,
        account
    }
}

function makeNonce(length) {
    var result           = '';
    var characters       = 'ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789';
    var charactersLength = characters.length;
    for ( var i = 0; i < length; i++ ) {
        result += characters.charAt(Math.floor(Math.random() * charactersLength));
    }
    return result;
}

function fixedEncodeURIComponent(str) {
    return encodeURIComponent(str).replace(/[\.!'()*]/g, function(c) {
      return '%' + c.charCodeAt(0).toString(16);
    });
}
