import { CognitoAccessToken, CognitoIdToken, CognitoRefreshToken, CognitoUser, CognitoUserPool, CognitoUserSession, ICognitoUserPoolData } from "amazon-cognito-identity-js";
import { useCallback } from "react";

const POOL_DATA = {
	UserPoolId: import.meta.env.VITE_USER_POOL_ID,
	ClientId: import.meta.env.VITE_COGNITO_CLIENT_ID,
} satisfies ICognitoUserPoolData

const userPool = new CognitoUserPool(POOL_DATA)

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

	const getCurrentUser = useCallback(async (): Promise<CognitoUser | null> => {
		const maybeUser = userPool.getCurrentUser()

		if (maybeUser === null) return null

		return new Promise((resolve, reject) => {
			maybeUser.getSession((_err: Error | null, ok: CognitoUserSession | null) => {
				if (!!ok) {
					// do tokens exist?
					if (ok?.isValid()) {
						// are tokens valid?
						maybeUser.getUserAttributes((err) => {
							if (!!err) {
								// we have an account, but it has expired tokens
								resolve(null)
							} else {
								// we have a valid account
								resolve(maybeUser)
							}
						})

						return;
					}
				}

				const originalSession = maybeUser.getSignInUserSession()

				if (originalSession === null) {
					console.warn("original session is null")
					return resolve(null)
				}

				maybeUser.refreshSession(originalSession.getRefreshToken(), (err, ok) => {
					if (!!err) return reject(err)
					resolve(ok)
				})
			})
		})
	}, [])

	const signOut = useCallback(async (): Promise<void> => {
		const user = userPool.getCurrentUser();

		if (!user) {
			console.warn("sign out request, but there is no user")

			return;
		}

		return new Promise((resolve, reject) => {
			user.getSession((err: Error | null, ok: CognitoUserSession | null) => {
				if (!!ok) {
					// do tokens exist?
					if (ok?.isValid()) {
						// are tokens valid?
						user.signOut()
						resolve()
					} else {
						reject(new Error('session invalid'))
					}
				} else {
					reject(err)
				}
			})
		})



	}, [])

	return {
		beginUserSession,
		getCurrentUser,
		signOut,
	}
}