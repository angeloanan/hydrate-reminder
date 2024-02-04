import { render } from "solid-js/web";

import "./styles.css";

const OAuthPage = () => {
  return (
    <>
      <h1>OAuth page</h1>
    </>
  )
}

render(() => <OAuthPage />, document.getElementById("root") as HTMLElement);
