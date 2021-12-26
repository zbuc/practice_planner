---
id: 'Button'
section: components
cssPrefix: pf-d-button
---## Examples

### Progress button - initial

```html
<form novalidate class="pf-c-form">
  <div class="pf-c-form__group">
    <div class="pf-c-form__group-label">
      <label class="pf-c-form__label" for="progress-button-initial-login">
        <span class="pf-c-form__label-text">Username</span>
        <span class="pf-c-form__label-required" aria-hidden="true">&#42;</span>
      </label>
    </div>
    <div class="pf-c-form__group-control">
      <input
        class="pf-c-form-control"
        type="text"
        id="progress-button-initial-login"
        name="progress-button-initial-login"
        value="johndoe"
        required
      />
    </div>
  </div>
  <div class="pf-c-form__group">
    <div class="pf-c-form__group-label">
      <label class="pf-c-form__label" for="progress-button-initial-password">
        <span class="pf-c-form__label-text">Password</span>
        <span class="pf-c-form__label-required" aria-hidden="true">&#42;</span>
      </label>
    </div>
    <div class="pf-c-form__group-control">
      <input
        class="pf-c-form-control"
        type="password"
        value="p@ssw0rd"
        id="progress-button-initial-password"
        name="progress-button-initial-password"
        required
      />
    </div>
  </div>
  <div class="pf-c-form__group pf-m-action">
    <div class="pf-c-form__actions">
      <button
        class="pf-c-button pf-m-primary"
        type="submit"
      >Link account and log in</button>
    </div>
  </div>
</form>

```

### Progress button - loading

```html
<form novalidate class="pf-c-form">
  <div class="pf-c-form__group">
    <div class="pf-c-form__group-label">
      <label class="pf-c-form__label" for="progress-button-loading-login">
        <span class="pf-c-form__label-text">Username</span>
        <span class="pf-c-form__label-required" aria-hidden="true">&#42;</span>
      </label>
    </div>
    <div class="pf-c-form__group-control">
      <input
        class="pf-c-form-control"
        type="text"
        id="progress-button-loading-login"
        name="progress-button-loading-login"
        value="johndoe"
        required
      />
    </div>
  </div>
  <div class="pf-c-form__group">
    <div class="pf-c-form__group-label">
      <label class="pf-c-form__label" for="progress-button-loading-password">
        <span class="pf-c-form__label-text">Password</span>
        <span class="pf-c-form__label-required" aria-hidden="true">&#42;</span>
      </label>
    </div>
    <div class="pf-c-form__group-control">
      <input
        class="pf-c-form-control"
        type="password"
        value="p@ssw0rd"
        id="progress-button-loading-password"
        name="progress-button-loading-password"
        required
      />
    </div>
  </div>
  <div class="pf-c-form__group pf-m-action">
    <div class="pf-c-form__actions">
      <button
        class="pf-c-button pf-m-progress pf-m-in-progress pf-m-primary"
        type="submit"
      >
        <span class="pf-c-button__progress">
          <span
            class="pf-c-spinner pf-m-md"
            role="progressbar"
            aria-label="Loading..."
          >
            <span class="pf-c-spinner__clipper"></span>
            <span class="pf-c-spinner__lead-ball"></span>
            <span class="pf-c-spinner__tail-ball"></span>
          </span>
        </span>
        Linking account
      </button>
    </div>
  </div>
</form>

```

### Progress button - complete

```html
<form novalidate class="pf-c-form">
  <div class="pf-c-form__group">
    <div class="pf-c-form__group-label">
      <label class="pf-c-form__label" for="progress-button-complete-login">
        <span class="pf-c-form__label-text">Username</span>
        <span class="pf-c-form__label-required" aria-hidden="true">&#42;</span>
      </label>
    </div>
    <div class="pf-c-form__group-control">
      <input
        class="pf-c-form-control"
        type="text"
        id="progress-button-complete-login"
        name="progress-button-complete-login"
        value="johndoe"
        required
      />
    </div>
  </div>
  <div class="pf-c-form__group">
    <div class="pf-c-form__group-label">
      <label class="pf-c-form__label" for="progress-button-complete-password">
        <span class="pf-c-form__label-text">Password</span>
        <span class="pf-c-form__label-required" aria-hidden="true">&#42;</span>
      </label>
    </div>
    <div class="pf-c-form__group-control">
      <input
        class="pf-c-form-control"
        type="password"
        value="p@ssw0rd"
        id="progress-button-complete-password"
        name="progress-button-complete-password"
        required
      />
    </div>
  </div>
  <div class="pf-c-form__group pf-m-action">
    <div class="pf-c-form__actions">
      <button class="pf-c-button pf-m-primary pf-m-start" type="submit">
        <span class="pf-c-button__icon pf-m-start">
          <i class="fas fa-check-circle" aria-hidden="true"></i>
        </span>
        Logged in
      </button>
    </div>
  </div>
</form>

```
