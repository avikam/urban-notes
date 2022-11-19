import { sign } from "./sign"
import sha from "sha.js";


describe('sign', () => {
    test("sanity sign", () => {
        const signature = sign('user@user.com', 'password', 'a=b')

        expect(signature.account).toEqual('user%40user%2ecom')

        expect(
            sha('sha256').update(`user@user.com_password_a=b_${signature.nonce}_${signature.timestamp}`).digest('base64')
        ).toStrictEqual(signature.signature);
    })
});
