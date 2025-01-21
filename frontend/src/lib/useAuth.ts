import { CognitoAccessToken, CognitoIdToken, CognitoRefreshToken, CognitoUser, CognitoUserAttribute, CognitoUserPool, CognitoUserSession, ICognitoUserPoolData } from "amazon-cognito-identity-js";
import { useCallback } from "react";

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

	return {
		userPool,
		beginUserSession,
	};
}