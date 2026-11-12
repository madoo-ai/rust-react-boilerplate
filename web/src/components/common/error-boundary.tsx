import { Component, type ErrorInfo, type ReactNode } from "react";

import { ServerErrorPage } from "@/pages/ServerErrorPage";

type State = {
  hasError: boolean;
};

export class ErrorBoundary extends Component<{ children: ReactNode }, State> {
  state: State = { hasError: false };

  static getDerivedStateFromError(): State {
    return { hasError: true };
  }

  componentDidCatch(error: Error, info: ErrorInfo) {
    console.error(error, info);
  }

  render() {
    if (this.state.hasError) {
      return (
        <ServerErrorPage onRetry={() => this.setState({ hasError: false })} />
      );
    }

    return this.props.children;
  }
}
