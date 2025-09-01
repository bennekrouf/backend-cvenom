#import "template.typ": conf, date, dated_experience, experience_details, section, get_text
#let get_work_experience() = {
  [
    == Company Name
    #dated_experience(
      "Job Title",
      date: "Start Date - End Date",
      description: "Brief company description",
      content: [
        #experience_details(
          "Key responsibility or achievement"
        )
        #experience_details(
          "Another responsibility or project"
        )
      ]
    )
    
    == Previous Company
    #dated_experience(
      "Previous Job Title",
      date: "Start Date - End Date",
      content: [
        #experience_details(
          "Previous role responsibility"
        )
      ]
    )
  ]
}
