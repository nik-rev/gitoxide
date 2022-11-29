#[cfg(any(
    feature = "blocking-http-transport-reqwest",
    feature = "blocking-http-transport-curl"
))]
mod http {
    use git_repository as git;
    use git_transport::client::http::options::{FollowRedirects, ProxyAuthMethod};

    pub(crate) fn repo(name: &str) -> git::Repository {
        let dir = git_testtools::scripted_fixture_repo_read_only("make_config_repos.sh").unwrap();
        git::open_opts(dir.join(name), git::open::Options::isolated()).unwrap()
    }

    fn http_options(repo: &git::Repository, remote_name: Option<&str>) -> git_transport::client::http::Options {
        let opts = repo
            .transport_options("https://example.com/does/not/matter", remote_name.map(Into::into))
            .expect("valid configuration")
            .expect("configuration available for http");
        opts.downcast_ref::<git_transport::client::http::Options>()
            .expect("http options have been created")
            .to_owned()
    }

    #[test]
    fn remote_overrides() {
        let repo = repo("http-remote-override");
        let git_transport::client::http::Options {
            proxy,
            proxy_auth_method,
            follow_redirects,
            ..
        } = http_options(&repo, Some("origin"));

        assert_eq!(proxy_auth_method, ProxyAuthMethod::Negotiate);
        assert_eq!(proxy.as_deref(), Some("http://overridden"));
        assert_eq!(follow_redirects, FollowRedirects::Initial);
    }

    #[test]
    fn simple_configuration() {
        let repo = repo("http-config");
        let git_transport::client::http::Options {
            extra_headers,
            follow_redirects,
            low_speed_limit_bytes_per_second,
            low_speed_time_seconds,
            proxy,
            no_proxy: _,
            proxy_auth_method,
            proxy_authenticate,
            user_agent,
            connect_timeout,
            verbose: _,
            backend,
        } = http_options(&repo, None);
        assert_eq!(
            extra_headers,
            &["ExtraHeader: value2", "ExtraHeader: value3"],
            "it respects empty values to clear prior values"
        );
        assert_eq!(follow_redirects, FollowRedirects::All);
        assert_eq!(low_speed_limit_bytes_per_second, 5120);
        assert_eq!(low_speed_time_seconds, 10);
        assert_eq!(proxy.as_deref(), Some("http://localhost:9090"),);
        assert!(
            proxy_authenticate.is_none(),
            "no username means no authentication required"
        );
        assert_eq!(proxy_auth_method, ProxyAuthMethod::Basic, "TODO: implement auth");
        assert_eq!(user_agent.as_deref(), Some("agentJustForHttp"));
        assert_eq!(connect_timeout, Some(std::time::Duration::from_millis(60 * 1024)));
        assert!(
            backend.is_none(),
            "backed is never set as it's backend specific, rather custom options typically"
        )
    }

    #[test]
    fn http_proxy_with_username() {
        let repo = repo("http-proxy-authenticated");

        let opts = http_options(&repo, None);
        assert_eq!(
            opts.proxy.as_deref(),
            Some("http://user@localhost:9090"),
            "usernames in proxy urls trigger authentication before making a connection…"
        );
        assert!(
            opts.proxy_authenticate.is_some(),
            "…and credential-helpers are used to do that. This could be overridden in remotes one day"
        );
        assert_eq!(
            opts.follow_redirects,
            FollowRedirects::All,
            "an empty value is true, so we can't take shortcuts for these"
        );
    }

    #[test]
    fn empty_proxy_string_turns_it_off() {
        let repo = repo("http-proxy-empty");

        let opts = http_options(&repo, None);
        assert_eq!(
            opts.proxy.as_deref(),
            Some(""),
            "empty strings indicate that the proxy is to be unset by the transport"
        );
        assert_eq!(opts.follow_redirects, FollowRedirects::None);
    }

    #[test]
    fn proxy_without_protocol_is_defaulted_to_http() {
        let repo = repo("http-proxy-auto-prefix");

        let opts = http_options(&repo, None);
        assert_eq!(opts.proxy.as_deref(), Some("http://localhost:9090"));
        assert_eq!(opts.follow_redirects, FollowRedirects::Initial);
    }
}
