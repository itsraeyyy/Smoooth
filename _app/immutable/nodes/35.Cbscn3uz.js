import{$ as e,F as t,Ft as n,Ht as r,It as i,K as a,L as o,M as s,St as c,Tt as l,V as u,Vt as d,_t as f,b as p,bt as m,ct as h,dt as g,f as _,gt as v,h as y,ht as b,l as x,mt as S,q as C,r as w,u as T,wt as E,x as D,z as O}from"../chunks/cBdug0a_.js";import"../chunks/xihTtKlq.js";import{$t as ee,E as k,H as A,It as te,K as j,V as M,W as ne,Z as N,b as P,gt as re,i as F,t as ie,wt as ae,x as I}from"../chunks/BK8tric1.js";import{d as L,i as R,t as z}from"../chunks/CrTMul68.js";import{n as B,t as V}from"../chunks/B-1wecjy.js";import{t as H}from"../chunks/DWcuEVfU.js";import{t as U}from"../chunks/BpWmU0gq.js";import{t as W}from"../chunks/qOYEBy4o.js";import{t as G}from"../chunks/BoHlb8qN.js";var K=`openapi: 3.1.0
info:
  title: Acme Billing API
  version: 2026-05-01
  description: Manage customers, invoices, subscriptions, and usage events.
servers:
  - url: https://api.acme.test/v1
security:
  - bearerAuth: []
tags:
  - name: Customers
    description: Customer records used for billing and lifecycle automation.
paths:
  /customers:
    get:
      tags:
        - Customers
      operationId: listCustomers
      summary: List customers
      description: Returns paginated customers ordered by creation date.
      parameters:
        - name: limit
          in: query
          description: Maximum number of customers to return.
          schema:
            type: integer
            default: 25
      responses:
        '200':
          description: A page of customers.
          content:
            application/json:
              schema:
                type: object
                properties:
                  data:
                    type: array
                    items:
                      $ref: '#/components/schemas/Customer'
              examples:
                success:
                  value:
                    data:
                      - id: cus_123
                        email: founder@example.com
        '401':
          description: Missing or invalid bearer token.
        '429':
          description: Rate limit exceeded.
  /customers/{customerId}:
    get:
      tags:
        - Customers
      operationId: getCustomer
      summary: Retrieve a customer
      description: Fetches one customer by stable customer ID.
      parameters:
        - name: customerId
          in: path
          required: true
          description: Stable customer identifier.
          schema:
            type: string
      responses:
        '200':
          description: Customer found.
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/Customer'
        '404':
          description: Customer not found.
components:
  securitySchemes:
    bearerAuth:
      type: http
      scheme: bearer
      bearerFormat: JWT
      description: Send a short-lived access token in the Authorization header.
  schemas:
    Customer:
      type: object
      description: A billing customer.
      required:
        - id
        - email
      properties:
        id:
          type: string
          description: Stable customer identifier.
          example: cus_123
        email:
          type: string
          format: email
          description: Customer email address.
          example: founder@example.com
externalDocs:
  description: API changelog
  url: https://acme.test/changelog
x-changelog:
  - date: 2026-05-01
    description: Added customer retrieval endpoint.`,q=u(`<span class="rounded-sm border border-border-strong px-2 py-0.5 font-mono text-xs text-fg-3"> </span>`),oe=u(`<p class="flex items-center gap-2.5 bg-rose-100 px-4 py-3 text-sm text-rose-ink"><span aria-hidden="true" class="block size-2 flex-none bg-rose"></span> </p>`),se=u(`<li class="flex flex-wrap items-baseline gap-x-2 gap-y-1 text-sm"><span class="flex shrink-0 items-center gap-1.5"><span aria-hidden="true"></span> <span class="text-2xs tracking-wide text-on-ink-faint uppercase"> </span></span> <span class="text-on-ink-muted"> </span> <span class="font-mono text-xs text-on-ink-faint"> </span></li>`),ce=u(`<ul class="mt-3 ml-8 space-y-2"></ul>`),le=u(`<div class="py-4"><div class="flex items-start justify-between gap-4"><div class="flex min-w-0 items-start gap-3"><!> <div class="min-w-0"><h4 class="text-md font-medium text-on-ink"> </h4> <p class="mt-1 text-sm text-pretty text-on-ink-muted"> </p></div></div> <span class="flex shrink-0 items-center gap-2"><span aria-hidden="true"></span> <span class="text-md font-medium tabular-nums text-on-ink"> </span></span></div> <!></div>`),ue=u(`<div class="mt-2 divide-y divide-ink-line"></div>`),de=u(`<div class="flex min-h-[280px] flex-col items-center justify-center text-center"><div class="size-7 animate-spin rounded-full border-2 border-ink-line border-t-spark motion-reduce:animate-none"></div> <p class="mt-4 text-sm text-on-ink-muted">Validating your spec…</p></div>`),fe=u(`<div class="flex min-h-[280px] flex-col items-center justify-center text-center"><!> <p class="mt-3 max-w-xs text-sm text-pretty text-on-ink-muted">Paste a spec, upload a file, or fetch a URL to see a full quality breakdown.</p></div>`),pe=u(`<div class="rounded-md border border-ink-line bg-ink-raised p-5"><div class="flex items-start justify-between gap-4"><div class="min-w-0"><p class="truncate text-sm text-on-ink-muted"> </p> <p class="mt-1 font-display text-display-md tabular-nums text-on-ink"> <span class="text-title text-on-ink-faint">/100</span></p></div> <span class="flex shrink-0 flex-col items-end gap-2"><span aria-hidden="true"></span> <span class="font-display text-title-lg text-on-ink"> </span></span></div> <p class="mt-3 text-sm text-pretty text-on-ink-muted"> </p> <div class="mt-5 grid grid-cols-3 gap-3 border-t border-ink-line pt-4"><div><p class="font-display text-title-lg tabular-nums text-on-ink"> </p> <p class="mt-1 flex items-center gap-1 text-2xs text-on-ink-muted"><span aria-hidden="true" class="block size-2 bg-rose"></span>Errors</p></div> <div><p class="font-display text-title-lg tabular-nums text-on-ink"> </p> <p class="mt-1 flex items-center gap-1 text-2xs text-on-ink-muted"><span aria-hidden="true" class="block size-2 bg-sand"></span>Warnings</p></div> <div><p class="font-display text-title-lg tabular-nums text-on-ink"> </p> <p class="mt-1 flex items-center gap-1 text-2xs text-on-ink-muted"><span aria-hidden="true" class="block size-2 bg-glacier"></span>Suggestions</p></div></div></div> <p class="mt-3 text-center text-sm tabular-nums text-on-ink-faint"> </p>`,1),me=u(`<section class="overflow-hidden rounded-md border border-border-strong bg-bg-raised"><div><div class="flex flex-wrap items-center justify-between gap-3 border-b border-border bg-bg-sunken px-4 py-3"><div class="flex items-center gap-2.5"><span class="grid size-7 place-items-center rounded bg-ink-900 font-mono text-xs font-medium text-spark"></span> <span class="text-base font-medium text-fg">OpenAPI specification</span> <!></div> <div class="flex flex-wrap items-center gap-2"><div class="flex items-center gap-1.5"><input type="url" placeholder="Fetch from a URL" aria-label="OpenAPI spec URL" class="input-face h-8 w-44 px-2.5 text-sm placeholder:text-fg-4 sm:w-52"/> <!></div> <!> <!> <input type="file" accept=".json,.yaml,.yml,application/json,application/yaml,text/yaml" class="sr-only"/></div></div> <textarea spellcheck="false" placeholder="Paste your OpenAPI or Swagger spec here, in JSON or YAML…" class="block h-[340px] w-full resize-y border-0 bg-ink-950 p-4 font-mono text-sm leading-6 text-on-ink placeholder:text-on-ink-faint focus:ring-0 md:h-[420px]" aria-label="OpenAPI JSON or YAML input"></textarea> <!></div> <div class="grid bg-ink-900 text-on-ink lg:grid-cols-[1.6fr_1fr]"><div class="p-5 md:p-6"><h3 class="eyebrow text-spark">Documentation quality</h3> <!></div> <aside class="border-t border-ink-line p-5 md:p-6 lg:border-t-0 lg:border-l"><!> <div class="mt-4 rounded-md border border-ink-line bg-ink-well p-5"><span class="eyebrow text-spark">Next step</span> <h4 class="mt-2 font-display text-title text-balance text-on-ink">A valid spec still goes stale</h4> <p class="mt-2 text-sm text-pretty text-on-ink-muted">This validator scores your spec today. Ferndesk keeps your API reference, guides, and
					changelog accurate as your endpoints change, for developers and AI agents alike.</p> <div class="mt-5 flex flex-wrap gap-2"><!> <!></div></div></aside></div></section>`);function J(a,u){i(u,!0);let x={syntax:N,completeness:A,examples:ee,auth:M,responses:te,changelogs:ne,"agent-readiness":ae},E=c(f(K)),L=c(``),R=c(`loading`),z=c(``),B=c(null),V,U=0,W,G=!1,J=!1;function Y(e){G||(G=!0,F(`Tool Started`,{tool:`openapi_validator`,method:e}))}let X=l(()=>{let t=e(B)?e(B).categories.flatMap(e=>e.issues):[];return{error:t.filter(e=>e.severity===`error`).length,warning:t.filter(e=>e.severity===`warning`).length,suggestion:t.filter(e=>e.severity===`suggestion`).length}});w(()=>{Z()});function he(){Y(`sample`),m(E,K,!0),m(L,``),Z()}async function ge(e){let t=e.currentTarget,n=t.files?.[0];n&&(Y(`upload`),m(E,await n.text(),!0),t.value=``,Z())}function _e(){Y(`paste`),V&&clearTimeout(V),V=setTimeout(()=>{Z()},500)}async function Z(){await ye({type:`raw`,rawSpec:e(E)})}async function ve(){let t=e(L).trim();t&&(Y(`url`),await ye({type:`url`,url:t}))}async function ye(e){if(e.type===`raw`&&!e.rawSpec.trim()){m(B,null),m(R,`idle`);return}let t=++U;m(R,`loading`),m(z,``);try{let n=await(await fetch(`/tools/openapi-validator/analyze`,{method:`POST`,headers:{"content-type":`application/json`},body:JSON.stringify(e)})).json();if(t!==U)return;if(!n.ok){m(B,null),m(z,n.error,!0),m(R,`error`);return}m(B,n.report,!0),n.rawSpec&&m(E,n.rawSpec,!0),m(R,`idle`),G&&!J&&(J=!0,F(`Tool Completed`,{tool:`openapi_validator`,score:n.report.weightedScore,grade:n.report.grade}))}catch(e){if(t!==U)return;m(B,null),m(z,e instanceof Error?e.message:`The server could not validate this OpenAPI document.`,!0),m(R,`error`)}}function be(e){return e>=80?`bg-fern-400`:e>=60?`bg-sand`:`bg-rose`}function xe(e){return e===`error`?`bg-rose`:e===`warning`?`bg-sand`:`bg-glacier`}var Se=me(),Ce=S(Se),we=S(Ce),Te=S(we),Ee=S(Te);Ee.textContent=`{}`;var De=v(Ee,4),Oe=t=>{var n=q(),i=S(n,!0);r(n),h(e=>o(i,e),[()=>e(B).format.toUpperCase()]),O(t,n)};t(De,t=>{e(B)&&t(Oe)}),r(Te);var ke=v(Te,2),Ae=S(ke),Q=S(Ae);y(Q);var je=v(Q,2);{let t=l(()=>e(R)===`loading`||!e(L).trim());H(je,{theme:`secondary`,size:`sm`,label:`Fetch`,get icon(){return re},get disabled(){return e(t)},onclick:ve})}r(Ae);var Me=v(Ae,2);H(Me,{theme:`secondary`,size:`sm`,label:`Upload`,get icon(){return j},onclick:()=>W.click()});var Ne=v(Me,2);H(Ne,{theme:`secondary`,size:`sm`,label:`Load sample`,onclick:he});var Pe=v(Ne,2);T(Pe,e=>W=e,()=>W),r(ke),r(we);var $=v(we,2);g($);var Fe=v($,2),Ie=t=>{var n=oe(),i=v(S(n));r(n),h(()=>o(i,` ${e(z)??``}`)),O(t,n)};t(Fe,t=>{e(R)===`error`&&e(z)&&t(Ie)}),r(Ce);var Le=v(Ce,2),Re=S(Le),ze=v(S(Re),2),Be=n=>{var i=ue();s(i,21,()=>e(B).categories,e=>e.id,(n,i)=>{var a=le(),c=S(a),l=S(c),u=S(l);ie(u,{get icon(){return x[e(i).id]},size:20,className:`mt-0.5 shrink-0 text-on-ink-faint`});var d=v(u,2),f=S(d),m=S(f,!0);r(f);var g=v(f,2),_=S(g,!0);r(g),r(d),r(l);var y=v(l,2),b=S(y),C=v(b,2),w=S(C,!0);r(C),r(y),r(c);var T=v(c,2),E=t=>{var n=ce();s(n,21,()=>e(i).issues,e=>e.path+e.message,(t,n)=>{var i=se(),a=S(i),s=S(a),c=v(s,2),l=S(c,!0);r(c),r(a);var u=v(a,2),d=S(u,!0);r(u);var f=v(u,2),m=S(f,!0);r(f),r(i),h(t=>{p(s,1,t),o(l,e(n).severity),o(d,e(n).message),o(m,e(n).path)},[()=>D([`block size-2`,xe(e(n).severity)])]),O(t,i)}),r(n),O(t,n)};t(T,t=>{e(i).issues.length&&t(E)}),r(a),h(t=>{o(m,e(i).label),o(_,e(i).whatItChecks),p(b,1,t),o(w,e(i).score)},[()=>D([`block size-2`,be(e(i).score)])]),O(n,a)}),r(i),O(n,i)},Ve=e=>{O(e,de())},He=e=>{var t=fe();ie(S(t),{get icon(){return N},size:28,className:`text-on-ink-faint`}),d(2),r(t),O(e,t)};t(ze,t=>{e(B)?t(Be):e(R)===`loading`?t(Ve,1):t(He,-1)}),r(Re);var Ue=v(Re,2),We=S(Ue),Ge=t=>{var n=pe(),i=b(n),a=S(i),s=S(a),c=S(s),l=S(c,!0);r(c);var u=v(c,2),f=S(u,!0);d(),r(u),r(s);var m=v(s,2),g=S(m),_=v(g,2),y=S(_,!0);r(_),r(m),r(a);var x=v(a,2),C=S(x,!0);r(x);var w=v(x,2),T=S(w),E=S(T),ee=S(E,!0);r(E),d(2),r(T);var k=v(T,2),A=S(k),te=S(A,!0);r(A),d(2),r(k);var j=v(k,2),M=S(j),ne=S(M,!0);r(M),d(2),r(j),r(w),r(i);var N=v(i,2),P=S(N);r(N),h(t=>{o(l,e(B).title),o(f,e(B).weightedScore),p(g,1,t),o(y,e(B).grade),o(C,e(B).verdict),o(ee,e(X).error),o(te,e(X).warning),o(ne,e(X).suggestion),o(P,`${e(B).pathCount??``} paths · ${e(B).methodCount??``} operations · ${e(B).schemaCount??``} schemas`)},[()=>D([`block size-3`,be(e(B).weightedScore)])]),O(t,n)};t(We,t=>{e(B)&&t(Ge)});var Ke=v(We,2),qe=v(S(Ke),6),Je=S(qe);H(Je,{theme:`spark`,size:`md`,label:`Start free`,get href(){return k},target:`_blank`,trackModule:`tool_next_step`}),H(v(Je,2),{theme:`outline`,size:`md`,label:`Book a demo`,get imageIcon(){return P},get href(){return I},target:`_blank`,trackModule:`tool_next_step`}),r(qe),r(Ke),r(Ue),r(Le),r(Se),C(`keydown`,Q,e=>e.key===`Enter`&&ve()),_(Q,()=>e(L),e=>m(L,e)),C(`change`,Pe,ge),C(`input`,$,_e),_($,()=>e(E),e=>m(E,e)),O(a,Se),n()}a([`keydown`,`change`,`input`]);var Y=u(`<li class="flex items-center gap-2.5 text-base font-medium text-fg"><span aria-hidden="true" class="block size-2 flex-none bg-fern-600"></span> </li>`),X=u(`<div class="rounded-md border border-border-strong bg-bg-raised p-6"><!> <h3 class="mt-4 font-display text-title-sm text-fg"> </h3> <p class="mt-2 text-base text-pretty text-prose"> </p></div>`),he=u(`<!> <!> <!> <main class="bg-bg"><section class="border-b border-border pt-14 pb-16 lg:pt-20 lg:pb-24"><div class="container-marketing"><div class="max-w-[720px]"><span class="eyebrow">OpenAPI validator</span> <h1 class="mt-4 font-display text-display-md text-balance lg:text-display-lg">Free OpenAPI &amp; Swagger validator</h1> <p class="mt-5 text-lede text-pretty text-prose">Paste, upload, or link an OpenAPI or Swagger spec and get an instant readiness score
					across syntax, completeness, examples, auth, responses, changelogs, and AI
					agent-readiness.</p> <ul class="mt-7 flex flex-col gap-2.5 sm:flex-row sm:flex-wrap sm:gap-x-8"></ul></div> <div class="mt-10"><!></div></div></section> <section class="border-b border-border py-16 lg:py-24"><div class="container-marketing"><div class="grid grid-cols-1 gap-14 lg:grid-cols-3 lg:gap-16"><div class="lg:sticky lg:top-24 lg:self-start"><span class="eyebrow">What it checks</span> <h2 class="mt-3 font-display text-display-sm text-balance lg:text-display-md">More than schema validation</h2> <p class="mt-5 text-md text-pretty text-prose">Most validators answer one question: will tooling accept this file? This checker also
						asks whether your API reference is complete enough for a developer, support agent, or AI
						coding tool to take the next step without guessing.</p></div> <div class="grid gap-4 sm:grid-cols-2 lg:col-span-2"></div></div></div></section> <section class="border-b border-border py-16 lg:py-24"><div class="container-marketing"><div class="grid grid-cols-1 gap-14 lg:grid-cols-3 lg:gap-16"><div class="lg:sticky lg:top-24 lg:self-start"><span class="eyebrow">Agent-ready docs</span> <h2 class="mt-3 font-display text-display-sm text-balance lg:text-display-md">Valid OpenAPI is the floor. Agent-readiness is the bar.</h2></div> <div class="space-y-5 lg:col-span-2"><p class="text-lede text-pretty text-prose">AI coding assistants, support agents, and workflow automations read API docs differently
						from humans. They need stable names, explicit input rules, response shapes, auth
						behavior, error states, and examples that remove ambiguity.</p> <p class="text-lede text-pretty text-prose">That is why this validator scores operation IDs, descriptions, examples, schemas, 401
						and 403 responses, 429 behavior, changelog links, and deprecation signals. Those details
						help agents call your API safely instead of guessing from a syntactically valid file.</p></div></div></div></section> <!></main> <!> <!>`,1);function ge(t,a){i(a,!1);let c=[{question:`What does this OpenAPI validator check?`,answer:`It checks syntax, required OpenAPI structure, documentation completeness, examples, authentication, response documentation, changelog signals, and AI agent readiness.`},{question:`Does it support Swagger 2.0?`,answer:`Yes. The validator detects top-level Swagger 2.0 documents and scores the same documentation quality categories, although the recommendations are optimized for modern OpenAPI 3.x specs.`},{question:`Is my OpenAPI file uploaded to Ferndesk?`,answer:`The spec is sent to a Ferndesk server endpoint for validation and URL fetching, but the tool does not store it. Server-side fetching lets us avoid browser CORS failures and apply public-URL safety checks.`},{question:`What is agent-ready API documentation?`,answer:`Agent-ready API docs are specific enough for AI coding tools and support agents to call the right endpoint safely: stable operation IDs, clear summaries, realistic examples, auth rules, error states, pagination, and rate-limit behavior.`},{question:`How is this different from Swagger Editor or a schema validator?`,answer:`Traditional validators focus on whether the spec can be parsed. This tool also scores whether developers and AI agents can understand and use the API without guessing.`}],l=`Free OpenAPI Validator | Swagger & Agent-Ready API Docs Checker`,u=`Validate OpenAPI and Swagger specs, then score syntax, completeness, examples, auth, response docs, changelogs, and AI agent readiness. Free, no signup.`,d=L({name:l,description:u,path:`/tools/openapi-validator`}),f=R([{name:`Home`,url:`/`},{name:`Tools`,url:`/tools/openapi-validator`},{name:`OpenAPI Validator`,url:`/tools/openapi-validator`}]),p={"@context":`https://schema.org`,"@type":`WebApplication`,name:`Free OpenAPI Validator`,alternateName:`Swagger and Agent-Ready API Docs Checker`,url:`https://ferndesk.com/tools/openapi-validator`,applicationCategory:`DeveloperApplication`,operatingSystem:`Web`,description:`Validate OpenAPI and Swagger specs, then score syntax, completeness, examples, auth, response docs, changelogs, and AI agent readiness.`,offers:{"@type":`Offer`,price:`0`,priceCurrency:`USD`},featureList:[`OpenAPI syntax validation`,`Swagger validation`,`Documentation completeness scoring`,`Examples scoring`,`Authentication documentation scoring`,`Response documentation scoring`,`Changelog scoring`,`AI agent-readiness scoring`],publisher:{"@type":`Organization`,name:`Ferndesk`,url:`https://ferndesk.com`}},m=[`Free to use, no signup`,`Your spec is never stored`,`JSON or YAML, OpenAPI 3.x and Swagger 2.0`],g=[{icon:N,title:`Syntax`,body:`Required top-level fields, the OpenAPI or Swagger version, paths, operations, and whether the document parses at all.`},{icon:A,title:`Completeness`,body:`API overview, server URLs, endpoint summaries, descriptions, parameters, tags, and reusable schemas.`},{icon:ee,title:`Examples`,body:`Request examples, response examples, and schema examples, the real payloads a developer can copy and run.`},{icon:M,title:`Authentication`,body:`Security schemes, global and operation-level requirements, OAuth and API key clarity, and auth failure states.`},{icon:te,title:`Response docs`,body:`2xx success responses, 4xx and 5xx errors, descriptions, content types, schemas, and empty-response rules.`},{icon:ne,title:`Changelogs`,body:`Versioning, release notes, deprecations, migration context, and links to the API change history.`},{icon:ae,title:`Agent-readiness`,body:`Stable operation IDs, unambiguous descriptions, examples, schema clarity, rate limits, and safe error handling.`}];x();var _=he(),y=b(_);G(y,{title:l,description:u});var C=v(y,2);{let t=E(()=>[d,f,p]);z(C,{get schema(){return e(t)}})}var w=v(C,2);V(w,{});var T=v(w,2),D=S(T),k=S(D),j=S(k),P=v(S(j),6);s(P,5,()=>m,e=>e,(t,n)=>{var i=Y(),a=v(S(i));r(i),h(()=>o(a,` ${e(n)??``}`)),O(t,i)}),r(P),r(j);var re=v(j,2);J(S(re),{}),r(re),r(k),r(D);var F=v(D,2),I=S(F),H=S(I),K=v(S(H),2);s(K,5,()=>g,e=>e.title,(t,n)=>{var i=X(),a=S(i);ie(a,{get icon(){return e(n).icon},size:20,className:`text-fg-3`});var s=v(a,2),c=S(s,!0);r(s);var l=v(s,2),u=S(l,!0);r(l),r(i),h(()=>{o(c,e(n).title),o(u,e(n).body)}),O(t,i)}),r(K),r(H),r(I),r(F),W(v(F,4),{title:`OpenAPI validator FAQ`,subtitle:`Practical answers for API teams, technical writers, and developers maintaining API docs.`,get items(){return c},includeSchema:!0}),r(T);var q=v(T,2);U(q,{module:`tool_page_conversion`}),B(v(q,2),{}),O(t,_),n()}export{ge as component};