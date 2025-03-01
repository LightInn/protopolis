# Contributing

Thanks for considering contributing! Please read this document to learn the various ways you can contribute to this project and how to go about doing it.

## Bug reports and feature requests

### Did you find a bug?

First, do a quick search to see whether your issue has already been reported.
If your issue has already been reported, please comment on the existing issue.

Otherwise, open a new GitHub issue. Be sure to include a clear title and description. The description should include as much relevant information as possible. The description should explain how to reproduce the erroneous behavior as well as the behavior you expect to see. Ideally, you would include a code sample or an executable test case demonstrating the expected behavior.

### Do you have a suggestion for an enhancement or new feature?

We use GitHub issues to track feature requests. Before you create a feature request:

* Make sure you have a clear idea of the enhancement you would like. If you have a vague idea, consider discussing it first on a GitHub issue.
* Check the documentation to make sure your feature does not already exist.
* Do a quick search to see whether your feature has already been suggested.

When creating your request, please:

* Provide a clear title and description.
* Explain why the enhancement would be useful. It may be helpful to highlight the feature in other libraries.
* Include code examples to demonstrate how the enhancement would be used.

## Making a pull request

When you're ready to contribute code to address an open issue, please follow these guidelines to help us be able to review your pull request (PR) quickly.

1. **Initial setup** (only do this once)

   If you haven't already done so, please fork this repository on GitHub.

   Then clone your fork locally with

        git clone https://github.com/USERNAME/protopolis.git

   or

        git clone git@github.com:USERNAME/protopolis.git

   At this point, the local clone of your fork only knows that it came from *your* repo, github.com/USERNAME/protopolis.git, but doesn't know anything about the *main* repo, https://github.com/LightInn/protopolis.git. You can see this by running

        git remote -v

   which will output something like this:

        origin https://github.com/USERNAME/protopolis.git (fetch)
        origin https://github.com/USERNAME/protopolis.git (push)

   This means that your local clone can only track changes from your fork, but not from the main repo, and so you won't be able to keep your fork up-to-date with the main repo over time. Therefore you'll need to add another "remote" to your clone that points to https://github.com/LightInn/protopolis.git. To do this, run the following:

        git remote add upstream https://github.com/LightInn/protopolis.git

   Now if you do `git remote -v` again, you'll see

        origin https://github.com/USERNAME/protopolis.git (fetch)
        origin https://github.com/USERNAME/protopolis.git (push)
        upstream https://github.com/LightInn/protopolis.git (fetch)
        upstream https://github.com/LightInn/protopolis.git (push)

2. **Ensure your fork is up-to-date**

   Once you've added an "upstream" remote pointing to https://github.com/LightInn/protopolis.git, keeping your fork up-to-date is easy:

        git checkout main  # if not already on main
        git pull --rebase upstream main
        git push

3. **Create a new branch to work on your fix or enhancement**

   Committing directly to the main branch of your fork is not recommended. It will be easier to keep your fork clean if you work on a separate branch for each contribution you intend to make.

   You can create a new branch with

        # replace BRANCH with whatever name you want to give it
        git checkout -b BRANCH
        git push -u origin BRANCH

4. **Test your changes**

   Our continuous integration (CI) testing runs a number of checks for each pull request on GitHub Actions. You can run most of these tests locally, which is something you should do *before* opening a PR to help speed up the review process and make it easier for us.

   First, you should run `cargo fmt` to make sure your code is formatted consistently. Many IDEs support code formatters as plugins, so you may be able to set up `cargo fmt` to run automatically every time you save.

   Our CI also uses `cargo clippy` to lint the code base and `cargo test` for testing. You should run both of these next with

        cargo clippy
        cargo test

   We also strive to maintain high test coverage, so most contributions should include additions to the unit tests. These tests are run with `cargo test`, which you can use to locally run any test modules that you've added or changed.

   For example, if you've fixed a bug in `src/ui.rs`, you can run the tests specific to that module with

        cargo test --test ui

   After all of the above checks have passed, you can now open a new GitHub pull request. Make sure you have a clear description of the problem and the solution, and include a link to relevant issues.

   We look forward to reviewing your PR!