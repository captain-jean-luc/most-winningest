# Most Winning-est
Determines who is the most winningest on the "[Last one to post wins](https://community.tulpa.info/thread-game-last-one-to-post-wins)" thread. Scrapes and parses the posts and then spits out a nice sorted list of username: time winning.

## How to use:

01. Determine what the last page is (when browsing as an anomomys user, so 10 posts per page). If the most recent post is 7543, then the page that post is on is 755. If the most recent post is 7540, then that will be the last post on page 754.
02. Run main.rb, which simply downloads the pages as html files and puts them in the `pages/` folder. It takes two arguments, the starting and ending page to download, eg:

		ruby main.rb 1 754

03. Run most-winning.rb with no arguments, it will automatically read all the `pages/*.html` files and calculate a leaderboard of most winningness and output to stdout. If you want to save it in a file just redirect like so:

		ruby most-winning.rb > most-winning.txt

	If you get errors about "negative delta", redownload the pages it mentions as well as the pages before and after. If you accidentally downloaded a page that doesn't exist, then it will be page 1 again and this will cause problems, simply delete the offending page from `pages/`
	
And you're done! Wasn't that easy? No? Too bad.

## Other things:

* `parse-dotinfo-page.rb` 

  This is the common library used to parse an html string into posts with username, post time, etc

* `parse-from-file.rb`

  A convenience program to run the parser on a single file and print the output. Mostly useful for debugging. Run like:

      ruby parse-from-file.rb pages/1.html

* `needing-updates.rb`

	Runs through each file in `pages/` and finds any that still have relative timestamps (eg "5 minutes ago"). These files should be re-downloaded after a few hours have passed and the forums decide to show the real timestamp.
