import { CognitoAccessToken, CognitoIdToken, CognitoRefreshToken, CognitoUser, CognitoUserAttribute, CognitoUserPool, CognitoUserSession, ICognitoUserPoolData } from "amazon-cognito-identity-js";
import { useCallback, useEffect } from "react";

const POOL_DATA = {
	UserPoolId: import.meta.env.VITE_USER_POOL_ID,
	ClientId: import.meta.env.VITE_COGNITO_CLIENT_ID,
} satisfies ICognitoUserPoolData;

const userPool = new CognitoUserPool(POOL_DATA);

export default function useHandler() {
	const beginUserSession = useCallback((username: string, accessToken: string, idToken: string, refreshToken: string) => {
		const userSession = new CognitoUserSession({
			AccessToken: new CognitoAccessToken({ AccessToken: accessToken }),
			IdToken: new CognitoIdToken({ IdToken: idToken }),
			RefreshToken: new CognitoRefreshToken({ RefreshToken: refreshToken }),
		})

		const cognitoUser = new CognitoUser({
			Username: username,
			Pool: userPool,
		})

		cognitoUser.setSignInUserSession(userSession)
	}, [])

	const getCurrentUser = useCallback(async () => {
		const maybeUser = userPool.getCurrentUser()

		if (maybeUser === null) return null;

		return new Promise((resolve, reject) => {
			maybeUser.getSession((err: Error | null, ok: CognitoUserSession | null) => {
				if (!err) {
					if (ok?.isValid()) {
						return resolve(maybeUser);
					}

					const originalSession = maybeUser.getSignInUserSession();

					if (originalSession === null) {
						return reject("original session is null")
					}

					maybeUser.refreshSession(originalSession.getRefreshToken(), (err, ok) => {
						if (!!err) reject(err);
						resolve(ok)
					})
				} else {
					reject(err)
				}
			});
		})
	}, [])

	return {
		beginUserSession,
		getCurrentUser,
	};
}