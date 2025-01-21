import { Component, Suspense, use, useCallback } from "react"
import { useSearchParams } from "react-router-dom"

type AuthLoadedProps = {
	auth: any
}

class AuthErrorBoundary extends Component<{ children: any }> {
	state = { hasError: false }

	constructor(props: { children: any }) {
		super(props)
		this.state = { hasError: false };
	}

	static getDerivedStateFromError(_error: any) {
		return { hasError: true };
	}

	render() {
		if (this.state.hasError) {
			return <div>
				Please sign in again, this auth flow has expired
			</div>
		}
		return this.props.children;
	}
}

function AuthLoaded(props: AuthLoadedProps) {
	const response = use(props.auth);

	return (
		<div>
			{response["access_token"]}
		</div>
	)
}

export default function Auth() {
	const [searchParams] = useSearchParams()

	const fetchToken = useCallback(async (code: string): Promise<any> => {
		console.log(code)
		try {
			const result = await fetch(`http://localhost:4000/auth/redirect?code=${code}`)
			if (result.ok) {
				console.log(result)
				const object = await result.json()
				console.log(object)
				return object
			}
			console.error(result)
		} catch (e) {
			console.error(e)
		}
	}, [])

	const code = searchParams.get("code");

	return (
		code !== null
			? <>
				<AuthErrorBoundary>
					<Suspense fallback={<b>Loading...</b>}>
						<AuthLoaded auth={fetchToken(code)} />
					</Suspense>
				</AuthErrorBoundary>
			</>
			: <b>missing code</b>
	)
}