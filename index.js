const fs = require("./dist/lib.js");

fs.generateStatement(
  JSON.stringify({
    transactions: [
      {
        amount: 100,
        member_no: "00019",
        town: "Litein",
        e_mail: "jk@gmail.com",
        allnames: "Japheth Kiptoo",
        post_address: "Utawala, Nairobi",
        gsm_no: "0724765149",
        descript: "MMF",
        security_code: "001",
        trans_id: 2324,
        trans_date: new Date(),
        account_no: "001-00019-001",
        taxamt: 0,
        trans_type: "WITHDRAWAL",
        running_balance: 1000,
        running_shares: 0,
        netamount: 0,
        mop: "M-PESA",
        currency: "KES",
        p_amount: 0,
        w_amount: 0,
        i_amount: 0,
        statement: "lorem blaaa blaaaa",
        price: 0,
        shares: 0,
      },
    ],
    pdf_name: "test",
  }),
  true
)
  .then(() => {
    console.log("success");
  })
  .catch((e) => {
    console.log(e);
  });
