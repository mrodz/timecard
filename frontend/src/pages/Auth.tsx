import { Suspense, use, useCallback, useEffect } from "react"
import { Link, useNavigate, useSearchParams } from "react-router-dom"

import useAuth, { InvalidCodeRedirectError } from "@/lib/useAuth"

import { buttonVariants } from "@/components/ui/button"
import { Card, CardContent, CardHeader } from "@/components/ui/card";
import { ProtectedRouteLoadingScreenReject, ProtectedRouteLoadingScreenSuccess } from "./Layout";
import AuthErrorBoundary from "@/components/AuthErrorBoundary";

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

type UserServerResponse = {
	username: string,
	user_attributes: Map<string, string | undefined>
}

type UserLoadedProps = {
	user: Promise<UserServerResponse | null>,
	auth: AuthServerResponse,
}

function UserLoaded(props: UserLoadedProps) {
	const { beginUserSession } = useAuth()
	const navigate = useNavigate()

	const user: UserServerResponse | null = use(props.user)

	beginUserSession(user!.username, props.auth.access_token, props.auth.id_token, props.auth.refresh_token)

	useEffect(() => {
		navigate('/dashboard');
	}, [])

	return (
		<Card>
			<CardHeader>
				Hi 🎉
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
			throw e
		}

		return null;
	}, [])


	return (
		<div>
			<Suspense fallback={<ProtectedRouteLoadingScreenSuccess message="Loading User" />}>
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
			} else {
				throw new InvalidCodeRedirectError()
			}
		} catch (e) {
			console.error(e)
			throw e
		}
	}, [])

	const code = searchParams.get("code");

	return (
		code !== null
			? <>
				<AuthErrorBoundary>
					<Suspense fallback={<ProtectedRouteLoadingScreenSuccess message="Creating User" />}>
						<AuthLoaded auth={fetchToken(code)} />
					</Suspense>
				</AuthErrorBoundary>
			</>
			: <ProtectedRouteLoadingScreenReject message="Request is missing code, please try logging in again" />
	)
}