import { CognitoUser, CognitoUserAttribute } from "amazon-cognito-identity-js"
import { useEffect, useState } from "react"

export default function useAttributes(user: CognitoUser) {
	const [attributes, setAttributes] = useState<Map<string, string>>(new Map())

	useEffect(() => {
		new Promise<Map<string, string>>((resolve, reject) => {
			user.getUserAttributes((err, ok) => {
				if (err !== undefined) {
					reject(err)
				} else {
					const out = new Map<string, string>()

					for (const kv of ok!) {
						out.set(kv.Name, kv.Value)
					}

					resolve(out)
				}
			})
		}).then(ok => {
			setAttributes(ok)
		})
	}, [])

	return [attributes]
}