import { Component, Suspense, use, useCallback, useEffect } from "react"
import { Link, useNavigate, useSearchParams } from "react-router-dom"

import useAuth from "@/lib/useAuth"

import { buttonVariants } from "@/components/ui/button"
import { Card, CardContent, CardHeader } from "@/components/ui/card";
import useAttributes from "@/lib/useAttributes";

type AuthServerResponse = {
	"access_token": string,
	"expires_in": number,
	"id_token": string,
	"refresh_token": string,
	"token_type": string
};

type AuthLoadedProps = {
	auth: Promise<AuthServerResponse | null>,
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

type UserServerResponse = {
	username: string,
	user_attributes: Map<string, string | undefined>
}

type UserLoadedProps = {
	user: Promise<UserServerResponse | null>,
	auth: AuthServerResponse,
}

function UserLoaded(props: UserLoadedProps) {
	const { beginUserSession, userPool } = useAuth()
	const navigate = useNavigate()

	const user: UserServerResponse | null = use(props.user)

	beginUserSession(user!.username, props.auth.access_token, props.auth.id_token, props.auth.refresh_token)

	useEffect(() => {
		const timeout = setTimeout(() => {
			navigate('/dashboard');
		}, 4_000);

		return () => clearTimeout(timeout);
	}, [])

	const cognitoUser = userPool.getCurrentUser();

	return (
		<Card>
			<CardHeader>
				Hi, {cognitoUser?.getUsername() ?? "friend"} 🎉
			</CardHeader>
			<CardContent>
				<p>
					You are now authenticated. You will be redirected shortly. If not, you may click this button and redirect yourself!
				</p>
				<Link className={buttonVariants({ variant: 'outline' })} to="/dashboard">Go to dashboard</Link>
			</CardContent>
		</Card>
	)
}

function AuthLoaded(props: AuthLoadedProps) {
	const response: AuthServerResponse | null = use(props.auth);

	const fetchUser = useCallback(async (): Promise<UserServerResponse | null> => {
		try {
			const result = await fetch(`http://localhost:4000/user`, {
				credentials: 'include'
			});

			if (result.ok) {
				const object = await result.json()
				return object
			}
			console.error(result)
		} catch (e) {
			console.log(e)
		}

		return null;
	}, [])


	return (
		<div>
			{JSON.stringify(response)}
			<Suspense fallback={<b>Loading user...</b>}>
				<UserLoaded auth={response!} user={fetchUser()} />
			</Suspense>
		</div>
	)
}

export default function Auth() {
	const [searchParams] = useSearchParams()

	const fetchToken = useCallback(async (code: string): Promise<AuthServerResponse | null> => {
		try {
			const result = await fetch(`http://localhost:4000/redirect?code=${code}`, {
				credentials: 'include'
			})
			if (result.ok) {
				const object = await result.json()
				return object
			}
			console.error(result)
		} catch (e) {
			console.error(e)
		}
		return null;
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